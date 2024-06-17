use crate::{command, spawn_command, CommandError};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};

type Result<T> = std::result::Result<T, CommandError>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CallerIdentity {
    pub user_id: String,
    pub account: String,
    pub arn: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    #[serde(rename = "pullRequestId")]
    pub id: String,

    pub title: String,
    pub description: Option<String>,

    #[serde(rename = "pullRequestStatus")]
    pub status: String,

    #[serde(rename = "pullRequestTargets")]
    pub targets: Vec<PullRequestTarget>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PullRequestTarget {
    #[serde(rename = "repositoryName")]
    pub repository: String,
    #[serde(rename = "sourceReference")]
    pub source: String,
    #[serde(rename = "destinationReference")]
    pub destination: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestResponse {
    pub pull_request: PullRequest,
}

#[derive(
    Hash,
    strum::AsRefStr,
    Debug,
    Clone,
    PartialEq,
    Eq,
    strum::Display,
    Serialize,
    Deserialize,
    strum::EnumString,
)]
pub enum PullRequestStatus {
    #[strum(serialize = "open")]
    Open,

    #[strum(serialize = "closed")]
    Close,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestsList {
    pub pull_request_ids: Vec<String>,
}

pub async fn refresh_auth_if_needed(profile: String) -> Result<String> {
    let mut auth_spinner = Spinner::new(Spinners::Dots, "Checking credentials...".into());

    if let Ok(CallerIdentity { arn, .. }) = get_caller_identity_no_cache(profile.clone()).await {
        return Ok(arn);
    }
    // self.author_arn = Some(arn.clone());

    auth_spinner.stop_and_persist("🔐", "Authentication needed!".into());

    let mut s = Spinner::new(Spinners::Dots, "Performing SSO Authentication".into());

    login(&profile).await?;

    s.stop_and_persist("✅", "Authenticated!".into());

    let identity = get_caller_identity_no_cache(profile.clone()).await?;

    Ok(identity.arn)
}

/// AWS API Methods
pub async fn login(profile: &str) -> Result<()> {
    spawn_command!(
        "aws",
        "sso",
        "login",
        "--color",
        "off",
        "--profile",
        profile
    )
    .map_err(CommandError::from_io)?
    .wait()
    .await
    .map_err(CommandError::from_io)?;

    Ok(())
}

#[cached]
pub async fn get_caller_identity(profile: String) -> Result<CallerIdentity> {
    let output = command!(
        "aws",
        "sts",
        "get-caller-identity",
        "--output",
        "json",
        "--color",
        "off",
        "--profile",
        &profile
    )
    .output()
    .await
    .map_err(CommandError::from_io)?;

    if output.status.success() {
        let string_value = String::from_utf8(output.stdout)?;
        let caller_identity: CallerIdentity =
            serde_json::from_str(&string_value).map_err(CommandError::from_serde)?;

        return Ok(caller_identity);
    }

    let error_message = String::from_utf8(output.stderr)?;
    Err(CommandError::IOError(error_message.to_string()))
}

#[cached]
pub async fn get_region(profile: String) -> Result<String> {
    let stdout = command!(
        "aws",
        "configure",
        "get",
        "region",
        "--color",
        "off",
        "--profile",
        &profile
    )
    .output()
    .await
    .map_err(CommandError::from_io)?
    .stdout;

    let region = String::from_utf8(stdout)?;
    let region = region.trim().to_string();

    Ok(region)
}

#[cached]
pub async fn get_pull_request(id: String, profile: String) -> Result<PullRequestResponse> {
    let stdout = command!(
        "aws",
        "codecommit",
        "get-pull-request",
        "--pull-request-id",
        &id,
        "--color",
        "off",
        "--output",
        "json",
        "--profile",
        &profile
    )
    .output()
    .await
    .map_err(CommandError::from_io)?
    .stdout;

    let raw_stdout = String::from_utf8(stdout)?;

    serde_json::from_str(&raw_stdout).map_err(CommandError::from_serde)
}

#[cached]
pub async fn list_pull_requests(
    repository: String,
    pull_request_status: PullRequestStatus,
    profile: String,
) -> Result<PullRequestsList> {
    let status = &pull_request_status.to_string();

    let stdout = command!(
        "aws",
        "codecommit",
        "list-pull-requests",
        "--repository",
        &repository,
        "--output",
        "json",
        "--pull-request-status",
        status,
        "--color",
        "off",
        "--profile",
        &profile
    )
    .output()
    .await
    .map_err(CommandError::from_io)?
    .stdout;

    let raw_output = String::from_utf8(stdout)?;

    serde_json::from_str(&raw_output).map_err(CommandError::from_serde)
}

#[cached]
pub async fn list_my_pull_requests(
    repository: String,
    pull_request_status: Option<PullRequestStatus>,
    author_arn: String,
    profile: String,
) -> Result<PullRequestsList> {
    let status = &pull_request_status.map(|s| s.to_string());
    let stdout: Vec<u8>;

    if let Some(status) = status {
        let out = command!(
            "aws",
            "codecommit",
            "list-pull-requests",
            "--repository",
            &repository,
            "--output",
            "json",
            "--pull-request-status",
            status,
            "--author-arn",
            &author_arn,
            "--color",
            "off",
            "--profile",
            &profile
        )
        .output()
        .await
        .map_err(CommandError::from_io)?;

        if out.stdout.is_empty() {
            let error_message = String::from_utf8(out.stderr)?;
            return Err(CommandError::IOError(error_message.to_string()));
        }

        stdout = out.stdout
    } else {
        let out = command!(
            "aws",
            "codecommit",
            "list-pull-requests",
            "--repository",
            &repository,
            "--output",
            "json",
            "--author-arn",
            &author_arn,
            "--color",
            "off",
            "--profile",
            &profile
        )
        .output()
        .await
        .map_err(CommandError::from_io)?;

        if out.stdout.is_empty() {
            let error_message = String::from_utf8(out.stderr)?;
            return Err(CommandError::IOError(error_message.to_string()));
        }

        stdout = out.stdout
    }

    let raw_output = String::from_utf8(stdout)?;
    serde_json::from_str(&raw_output).map_err(CommandError::from_serde)
}

pub async fn create_pull_request(
    repository: String,
    title: String,
    description: String,
    source_branch: String,
    target_branch: String,
    profile: String,
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
        &title,
        "--description",
        &description,
        "--targets",
        &targets,
        "--color",
        "off",
        "--profile",
        &profile
    )
    .output()
    .await
    .map_err(CommandError::from_io)?
    .stdout;

    let string_output = String::from_utf8(stdout)?;

    serde_json::from_str(&string_output).map_err(CommandError::from_serde)
}

pub async fn merge_pr_by_squash(
    id: String,
    repository: String,
    message: String,
    name: String,
    email: String,
    profile: String,
) -> Result<PullRequest> {
    let output = command!(
        "aws",
        "codecommit",
        "merge-pull-request-by-squash",
        "--pull-request-id",
        &id,
        "--repository-name",
        &repository,
        "--commit-message",
        &message,
        "--author-name",
        &name,
        "--email",
        &email,
        "--profile",
        &profile,
        "--color",
        "off",
        "--output",
        "json"
    )
    .output()
    .await
    .map_err(CommandError::from_io)?;

    let raw_stdout = String::from_utf8(output.stdout)?;

    if let Ok(json) =
        serde_json::from_str::<PullRequestResponse>(&raw_stdout).map_err(CommandError::from_serde)
    {
        return Ok(json.pull_request);
    }

    let raw_stderr = String::from_utf8(output.stderr)?;
    Err(CommandError::IOError(raw_stderr))
}

pub async fn start_pipeline_execution(name: String, profile: String) -> Result<()> {
    command!(
        "aws",
        "codepipeline",
        "start-pipeline-execution",
        "--name",
        &name,
        "--profile",
        &profile,
        "--output",
        "json",
        "--color",
        "off"
    )
    .output()
    .await
    .map_err(CommandError::from_io)?;

    Ok(())
}
