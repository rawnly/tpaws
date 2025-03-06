use std::collections::HashMap;

use color_eyre::eyre::OptionExt;
use ai::groq::{ self, models };
use arboard::Clipboard;
use color_eyre::Result;
use colored::*;
use commands::{aws, git};
use config::DEFAULT_AI_MODEL;
use inquire::{Confirm, Select, Text};
use spinners::{Spinner, Spinners};

use crate::{cli, context::GlobalContext, costants, utils};

pub async fn create(
    ctx: &mut GlobalContext,
    args: cli::Args,
    title: Option<String>,
    description: Option<String>,
    base: String,
    slack: bool,
    copy: bool,
    ai_enhance: bool,
    ai_model: Option<String>,
) -> Result<()> {
    let model = ai_model.unwrap_or(DEFAULT_AI_MODEL.into());
    let target_process_url = target_process::get_base_url();
    let raw_region = aws::get_region(ctx.profile.clone()).await?;
    let region = raw_region.trim().to_string();

    let branch = git::current_branch_v2().await?.to_string();

    let feature_name = branch.split('/').last().unwrap_or(&branch);
    let tp_link = format!("{target_process_url}/entity/{feature_name}");

    let is_valid_tp_branch = utils::get_ticket_id_from_branch(branch.clone()).is_some();

    let mut title = match utils::grab_title(title, branch.clone()).await {
        Ok(v) => v,
        Err(_) => Text::new("Title:")
            .with_placeholder("Your PR Title")
            .prompt()?,
    };

    let mut description = if is_valid_tp_branch {
        description.unwrap_or(format!("See: {tp_link}"))
    } else {
        description.unwrap_or(Text::new("Description:").prompt()?)
    };

    if is_valid_tp_branch && ai_enhance {

        let api_key = utils::get_groq_api_key_or_prompt(&mut ctx.config)?;
        let ai_client = groq::Client::new(&api_key);

        let id = utils::get_ticket_id_from_branch(branch.clone()).ok_or_eyre("Invalid branch name. Cannot extract id")?;
        let assignable = target_process::get_assignable(id).await?;

        let messages = vec![models::Message::system(
            r#"
You're an AI created to help us generate a PR description given a UserStory or Bug. 
You will be provided with Title, Description, ID, URL and other details about the UserStory or Bug.


The title must be written in present tense and the description should be a brief summary of the changes.
Use UserStory/Bug info to improve title and description.o
Don't just copy the title/description, but provide a meaningful content.


Return a json object with the following fields:
{
    "title": "PR title",
    "description": "Pr description"
}

Return the json data as response. Just that, no other information is needed.
                "#
                .to_string(),
        ), 
            models::Message::user(serde_json::to_string_pretty(&assignable)?)];

        let response = ai_client.chat(groq::ChatPayload::new(&model, messages)).await?;
        let content = &response.choices.first().ok_or_eyre("Invalid AI response. No choices returned.")?.message.content;
        let payload = serde_json::from_str::<HashMap<String, String>>(content)?;

        title = payload.get("title").unwrap_or(&title).to_string();

        if let Some(d) = payload.get("description") {
            description = d.to_string();
            description += &format!("\nSee: {tp_link}")
        }
    }

    let base_branch = base;

    let repository = {
        let raw_url = git::get_remote_url("origin").await?;
        let url = raw_url.trim();

        match url.split('/').last() {
            Some(url) => url.trim().replace(".git", ""),
            None => "".into(),
        }
    };

    println!();
    println!("Check if the details below before proceding:");

    println!();
    println!("Title: {}", title.yellow());
    println!("Description: {}", description.trim().yellow());
    println!("Source Branch: {}", branch.trim().yellow());
    println!("Target Branch: {}", base_branch.trim().yellow());
    println!("Repository: {}", repository.yellow());

    println!();

    if !Confirm::new("Do you confirm?")
        .with_default(false)
        .prompt()?
    {
        println!("Operation aborted.");
        return Ok(());
    };

    if args.dry_run {
        return Ok(());
    }

    let mut pr_spinner = Spinner::new(Spinners::Dots, "Creating PR ...".into());

    let pr = aws::create_pull_request(
        repository.clone(),
        title.clone(),
        description,
        branch,
        base_branch,
        ctx.profile.clone(),
    )
    .await?;

    let pr_link = format!("https://{region}.console.aws.amazon.com/codesuite/codecommit/repositories/{repository}/pull-requests/{pr_id}/details", pr_id = pr.pull_request.id);

    pr_spinner.stop_and_persist("🔗", format!("PR Available at: {pr_link}"));

    if slack {
        if let Ok(slack_user_id) = std::env::var("SLACK_USER_ID") {
            println!();

            let user = select_user()?;
            println!("Reviewer: {}", user.name.yellow());

            let mut slack_spinner = Spinner::new(Spinners::Dots, "Sending slack message".into());

            slack::send_message(
            format!(
                "<@{slack_user_id}> opened a PR to: <@{reviewer}> - `{repository}` <{pr_link}|{pr_id}: {title}>",
                reviewer = user.id,
                pr_id = pr.pull_request.id,
            ),
            pr_link.clone(),
            tp_link
        ).await?;

            slack_spinner.stop_with_symbol("✅");
        }
    }

    if copy {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(pr_link.clone())?;

        println!("PR link copied to clipboard!");
    }

    Ok(())
}

fn select_user() -> Result<costants::User> {
    let name = Select::new(
        "Who is your reviewer?",
        costants::USERS.iter().map(|u| u.name).collect(),
    )
    .prompt()?;

    let user = costants::USERS.iter().find(|user| user.name == name);

    match user {
        Some(user) => Ok(user.clone()),
        None => panic!("invalid value provided: {}", name),
    }
}
