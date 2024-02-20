use clap::Parser;
use color_eyre::Result;
use colored::*;
use human_panic::setup_panic;
use inquire::{Confirm, Select};
use spinners::{Spinner, Spinners};
use std::process::Stdio;
use tokio::process::Command;

mod cli;
mod costants;
mod errors;

#[tokio::main]
#[allow(unreachable_code, unused_variables)]
async fn main() -> Result<()> {
    setup_panic!();

    #[cfg(debug_assertions)]
    color_eyre::install()?;

    if !is_installed("aws") {
        println!("Useful links:");
        println!(
            "- {}",
            "https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html"
                .yellow()
        );
        println!();

        return Ok(());
    }

    let args = cli::Args::parse();

    println!("checking credentials...");
    let output = Command::new("aws")
        .arg("sts")
        .arg("get-caller-identity")
        .output()
        .await?;

    if !output.status.success() {
        println!("Authenticating...");
        Command::new("aws").arg("sso").arg("login").output().await?;
        println!("Authentication completed!");
    }

    let region = String::from_utf8(
        Command::new("aws")
            .arg("configure")
            .arg("get")
            .arg("region")
            .output()
            .await?
            .stdout,
    )?
    .trim_end();

    let branch = String::from_utf8(
        Command::new("git")
            .arg("branch")
            .arg("--show-current")
            .output()
            .await?
            .stdout,
    )?;

    let repository = {
        let url = String::from_utf8(
            Command::new("git")
                .arg("remote")
                .arg("get-url")
                .arg("origin")
                .output()
                .await?
                .stdout,
        )?;

        match url.split('/').last() {
            Some(url) => url.trim().replace(".git", ""),
            None => "".into(),
        }
    };

    let feature_name = branch.split('/').last().unwrap_or_default();
    let tp_link = format!("https://satispay.tpondemand.com/entity/{feature_name}");

    // create the PR
    let confirmed = prompt_recap(&args, feature_name.trim(), branch.trim_end(), &repository)?;

    if !confirmed {
        println!("Operation aborted.");
        return Ok(());
    }

    let mut spinner = Spinner::new(Spinners::Dots, "Creating PR ...".into());

    let base_branch = args.base;

    if !args.dry_run {
        let pr_output = Command::new("aws")
        .arg("codecommit")
        .arg("create-pull-request")
        .arg("--title")
        .arg(args.title)
        .arg("--description")
        .arg(format!("See: {}", tp_link))
        .arg("--targets")
        .arg(format!("repositoryName={repository},sourceReference={branch},destinationReference={base_branch}"))
        .arg("--profile")
        .arg(args.profile)
        .output()
        .await?;

        if !pr_output.status.success() {
            let error = String::from_utf8(pr_output.stderr)?;

            spinner.stop_and_persist("FATAL", error);

            return Ok(());
        }
    }

    spinner.stop_with_message("PR Created".into());

    if !args.no_slack && !args.dry_run {
        // Check for slack things
        let slack_user_id = std::env::var("SLACK_USER_ID");

        if let Ok(slack_user_id) = std::env::var("SLACK_USER_ID") {
            println!();

            let user = select_user()?;
            println!("Reviewer: {}", user.name.yellow());

            send_message().await?;
        } else {
            println!();
        }
    }

    Ok(())
}

fn prompt_recap(
    args: &cli::Args,
    feature: &str,
    target_branch: &str,
    repository: &str,
) -> Result<bool> {
    let tp_link = format!("https://satispay.tpondemand.com/entity/{feature}");

    println!();
    println!("Check if the details below before proceding:");

    println!();
    println!("TP Link: {}", tp_link.blue());

    println!();
    println!("Title: {}", args.title);
    println!("Description: {}", format!("'See: {}'", tp_link).yellow());
    println!("Source Branch: {}", args.base.yellow());
    println!("Target Branch: {}", target_branch.yellow());
    println!("Repository: {}", repository.yellow());

    println!();

    Ok(Confirm::new("Do you confirm?")
        .with_default(false)
        .prompt()?)
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

async fn send_message() -> Result<()> {
    todo!("not yet implemented");
}

fn is_installed(command: &str) -> bool {
    Command::new(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}
