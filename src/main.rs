use clap::Parser;
use color_eyre::Result;
use colored::*;
use derive_setters::Setters;
use human_panic::setup_panic;
use inquire::{Confirm, Select};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use serde::Serialize;
use spinners::{Spinner, Spinners};
use std::{process::Stdio, time::Duration};
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

    if is_installed("aws") {
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

    let mut auth_spinner = Spinner::new(Spinners::Dots, "Checking credentials...".into());
    let output = Command::new("aws")
        .arg("sts")
        .arg("get-caller-identity")
        .output()
        .await?;

    if !output.status.success() {
        auth_spinner.stop_and_persist("🔐", "Authentication needed!".into());

        let mut s = Spinner::new(Spinners::Dots, "Performing SSO Authentication".into());
        Command::new("aws").arg("sso").arg("login").output().await?;
        s.stop_and_persist("✅", "Authenticated!".into());
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

    let mut pr_spinner = Spinner::new(Spinners::Dots, "Creating PR ...".into());

    let base_branch = args.base;

    if !args.dry_run {
        let pr_output = Command::new("aws")
        .arg("codecommit")
        .arg("create-pull-request")
        .arg("--output")
        .arg("json")
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

            pr_spinner.stop_and_persist("FATAL", error);

            return Ok(());
        }

        // let pr : PullRequestCreatedResponse = serde_json::from_str(pr_output);

        pr_spinner.stop_and_persist("✅", "Created!".into());

        if !args.no_slack {
            // Check for slack things
            let slack_user_id = std::env::var("SLACK_USER_ID");

            if let Ok(slack_user_id) = std::env::var("SLACK_USER_ID") {
                println!();

                let user = select_user()?;
                println!("Reviewer: {}", user.name.yellow());

                let mut slack_spinner =
                    Spinner::new(Spinners::Dots, "Sending slack message".into());
                send_message().await?;
                slack_spinner.stop_with_symbol("✅");
            }
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

async fn send_message() -> Result<reqwest::Response> {
    let headers: HeaderMap = {
        let mut h = HeaderMap::new();

        h.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        h
    };

    let client = reqwest::ClientBuilder::new()
        .https_only(true)
        .default_headers(headers)
        .timeout(Duration::from_secs(5))
        .build()?;

    let url = "https://hooks.slack.com/services/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";

    let payload = SlackMessage::default().blocks(vec![
        SlackMessageBlock::section(SlackMessageBlockType::Mrkdwn, ""),
        SlackMessageBlock::divider(),
        SlackMessageBlock::actions(vec![
            ButtonBlock::link("", "Code Commit"),
            ButtonBlock::link("", "Target Process"),
        ]),
    ]);

    let response = client.post(url).json(&payload).send().await?;

    Ok(response)
}

fn is_installed(command: &str) -> bool {
    Command::new(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

#[derive(serde::Serialize, Default, Setters, Debug, Clone)]
struct SlackMessage {
    blocks: Vec<SlackMessageBlock>,
}

#[allow(dead_code)]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum SlackMessageBlockType {
    Actions,
    Text,
    Section,
    Divider,
    Mrkdwn,
    PlainText,
    Button,
}

impl Default for SlackMessageBlockType {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(serde::Serialize, Default, Setters, Debug, Clone)]
struct SlackMessageBlock {
    #[serde(rename = "type")]
    _type: SlackMessageBlockType,
    text: Option<TextBlock>,
    elements: Option<Vec<ButtonBlock>>,
}

impl SlackMessageBlock {
    fn section(t: SlackMessageBlockType, text: &str) -> Self {
        Self {
            _type: SlackMessageBlockType::Text,
            text: Some(TextBlock {
                _type: t,
                text: text.into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn divider() -> Self {
        Self {
            text: None,
            _type: SlackMessageBlockType::Divider,
            ..Default::default()
        }
    }

    fn actions(items: Vec<ButtonBlock>) -> Self {
        Self {
            _type: SlackMessageBlockType::Actions,
            elements: Some(items),
            ..Default::default()
        }
    }
}

#[derive(serde::Serialize, Default, Setters, Debug, Clone)]
struct TextBlock {
    #[serde(rename = "type")]
    _type: SlackMessageBlockType,
    text: String,
    emoji: bool,
}

impl TextBlock {
    fn plain(content: &str) -> Self {
        Self {
            _type: SlackMessageBlockType::PlainText,
            text: content.into(),
            ..Default::default()
        }
    }

    fn markdown(content: &str) -> Self {
        Self {
            _type: SlackMessageBlockType::Mrkdwn,
            text: content.into(),
            ..Default::default()
        }
    }
}

#[derive(serde::Serialize, Setters, Default, Debug, Clone)]
struct ButtonBlock {
    #[serde(rename = "type")]
    _type: SlackMessageBlockType,
    url: Option<String>,
    text: TextBlock,
}

impl ButtonBlock {
    fn link(href: &str, label: &str) -> Self {
        Self {
            _type: SlackMessageBlockType::Button,
            url: Some(href.into()),
            text: TextBlock::plain(label),
        }
    }
}
