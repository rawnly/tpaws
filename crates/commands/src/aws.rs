use crate::{command, spawn_command};
use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CallerIdentity {
    pub user_id: String,
    pub account: String,
    pub arn: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub title: String,
    pub pull_request_id: String,
    pub pull_request_status: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestResponse {
    pub pull_request: PullRequest,
}

pub async fn login(profile: &str) -> Result<()> {
    spawn_command!("aws", "sso", "login", "--profile", profile)?
        .wait()
        .await?;

    Ok(())
}

pub async fn get_caller_identity() -> Result<CallerIdentity> {
    let output = command!("aws", "sts", "get-caller-identity", "--output", "json")
        .output()
        .await?;

    if output.status.success() {
        let string_value = String::from_utf8(output.stdout)?;
        let caller_identity: CallerIdentity = serde_json::from_str(&string_value)?;

        return Ok(caller_identity);
    }

    let error_message = String::from_utf8(output.stderr)?;

    Err(eyre!("{error_message}"))
}

pub async fn get_region() -> Result<String> {
    let stdout = command!("aws", "configure", "get", "region")
        .output()
        .await?
        .stdout;
    let region = String::from_utf8(stdout)?;

    Ok(region)
}

pub async fn create_pull_request(
    repository: &str,
    title: &str,
    description: &str,
    source_branch: &str,
    target_branch: &str,
    profile: &str,
) -> Result<PullRequestResponse> {
    let targets = format!(
        "repositoryName={},sourceReference={},destinationReference={}",
        repository, source_branch, target_branch
    );

    let stdout = command!(
        "aws",
        "codecommit",
        "create-pull-request",
        "--output",
        "json",
        "--title",
        title,
        "--description",
        description,
        "--targets",
        &targets,
        "--profile",
        profile
    )
    .output()
    .await?
    .stdout;

    let string_output = String::from_utf8(stdout)?;

    Ok(serde_json::from_str(&string_output)?)
}
