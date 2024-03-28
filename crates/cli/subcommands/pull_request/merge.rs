use color_eyre::Result;
use colored::*;
use commands::{aws, git};
use spinners::{Spinner, Spinners};
use target_process::models::EntityStates;

use crate::{
    context::GlobalContext,
    utils::{self, RepoMetadata},
};

pub async fn merge(
    ctx: GlobalContext,
    id: Option<String>,
    author: Option<String>,
    commit_message: Option<String>,
    region: String,
    metadata: RepoMetadata,
    quiet: bool,
) -> Result<()> {
    let RepoMetadata { repository, branch } = metadata;

    let pr = utils::get_pr_id(ctx.profile.clone(), id).await?;
    let link = utils::build_pr_link(region, repository.clone(), pr.id.to_string());

    let email = git::config("user.email".to_string())
        .await
        .unwrap_or(author.clone().unwrap_or(ctx.config.pr_email));

    let name = git::config("user.name".to_string())
        .await
        .unwrap_or(author.unwrap_or(ctx.config.pr_name));

    println!("Found 1 matching PR");
    println!();
    println!("[{}] {}", pr.id.yellow(), pr.title.yellow());
    println!("{}", pr.description.yellow());
    println!("{}", link.blue());
    println!();

    for target in pr.targets {
        println!(
            "From: {}",
            target.source.replace("refs/heads/", "").magenta()
        );
        println!(
            "To:   {}",
            target.destination.replace("refs/heads/", "").magenta()
        );
        println!();
    }

    if !inquire::Confirm::new("Are the info above correct?")
        .with_default(false)
        .prompt()?
    {
        println!("operation aborted.");
        return Ok(());
    }

    // TODO: Refactor once we have settings
    let name = if quiet {
        name
    } else {
        inquire::Text::new("Author Name")
            .with_default(&name)
            .prompt()?
    };

    // TODO: Refactor once we have settings
    let email = if quiet {
        email
    } else {
        inquire::Text::new("Author Email")
            .with_default(&email)
            .prompt()?
    };

    let commit = if quiet {
        commit_message.unwrap_or(pr.description)
    } else {
        inquire::Text::new("Commit Message")
            .with_default(&commit_message.unwrap_or(pr.description))
            .prompt()?
    };

    if inquire::Confirm::new("Confirm?")
        .with_default(false)
        .prompt()?
    {
        let mut merge_spinner = Spinner::new(Spinners::Dots, format!("Squashing {}...", pr.id));

        let updated_pr =
            aws::merge_pr_by_squash(pr.id, repository, commit, name, email, ctx.profile).await?;

        merge_spinner.stop_with_symbol("✅");

        if inquire::Confirm::new("Delete remote branch?")
            .with_default(true)
            .prompt()?
        {
            let mut branch_spinner =
                Spinner::new(spinners::Spinners::Dots, "Deleting branch...".into());
            git::delete_remote_branch("origin", branch.clone()).await?;
            git::fetch(true).await?;
            branch_spinner.stop_with_symbol("✅");
        }

        let mut spinner = Spinner::new(
            spinners::Spinners::Dots,
            "Updating ticket status..".to_string(),
        );

        if let Some(assignable_id) = utils::get_ticket_id_from_branch(branch) {
            // TODO: remove this api call and parse assignable_id to usize
            let ticket = target_process::get_assignable(assignable_id).await?;

            if ticket.is_user_story() {
                target_process::update_entity_state(ticket.id, EntityStates::InStaging).await?;

                spinner.stop_with_symbol("✅");
            } else {
                spinner.stop_and_persist("⏩", "Skipped".to_string());
            }
        } else {
            spinner.stop_and_persist("✅", "Failed to update ticket status".to_string());
        }

        dbg!(&updated_pr);

        return Ok(());
    }

    println!("operation aborted.");

    Ok(())
}
