use color_eyre::Result;
use colored::*;
use commands::{aws, git};
use inquire::{Confirm, Select, Text};
use spinners::{Spinner, Spinners};

use crate::{cli, context::GlobalContext, costants, utils};

pub async fn create(
    ctx: GlobalContext,
    args: cli::Args,
    title: Option<String>,
    description: Option<String>,
    base: String,
    slack: bool,
) -> Result<()> {
    let target_process_url = target_process::get_base_url();
    let raw_region = aws::get_region(ctx.profile.clone()).await?;
    let region = raw_region.trim().to_string();

    let raw_branch = git::current_branch().await?;
    let branch = raw_branch.trim().to_string();

    let feature_name = branch.split('/').last().unwrap_or(&branch);
    let tp_link = format!("{target_process_url}/entity/{feature_name}");

    let is_valid_tp_branch = utils::get_ticket_id_from_branch(branch.clone()).is_some();

    let title = match utils::grab_title(title, branch.clone()).await {
        Ok(v) => v,
        Err(_) => Text::new("Title:")
            .with_placeholder("Your PR Title")
            .prompt()?,
    };

    let description = if is_valid_tp_branch {
        description.unwrap_or(format!("See: {tp_link}"))
    } else {
        description.unwrap_or(Text::new("Description:").prompt()?)
    };

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
        ctx.profile,
    )
    .await?;

    let pr_link = format!("https://{region}.console.aws.amazon.com/codesuite/codecommit/repositories/{repository}/pull-requests/{pr_id}/details", pr_id = pr.pull_request.id);

    pr_spinner.stop_and_persist("🔗", format!("PR Available at: {pr_link}"));

    if !slack {
        return Ok(());
    }

    if let Ok(slack_user_id) = std::env::var("SLACK_USER_ID") {
        println!();

        let user = select_user()?;
        println!("Reviewer: {}", user.name.yellow());

        let mut slack_spinner = Spinner::new(Spinners::Dots, "Sending slack message".into());

        let pr_link = format!("https://{region}.console.aws.amazon.com/codesuite/codecommit/repositories/{repository}/pull-requests/{pr_id}/details", pr_id = pr.pull_request.id);

        slack::send_message(
            format!(
                "<@{slack_user_id}> opened a PR to: <@{reviewer}> - `{repository}` <{pr_link}|{pr_id}: {title}>",
                reviewer = user.id,
                pr_id = pr.pull_request.id,
            ),
            pr_link,
            tp_link
        ).await?;

        slack_spinner.stop_with_symbol("✅");
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
