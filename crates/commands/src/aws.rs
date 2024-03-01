use crate::{command, spawn_command};
use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};

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
    pub description: String,

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

#[derive(strum::Display)]
pub enum PullRequestStatus {
    Open,
    Close,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestsList {
    pub pull_request_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AWS {
    pub profile: String,
    pub region: Option<String>,
}

impl AWS {
    pub fn new(profile: String) -> Self {
        Self {
            profile,
            region: None,
        }
    }

    pub async fn refresh_auth_if_needed(&self) -> Result<()> {
        let mut auth_spinner = Spinner::new(Spinners::Dots, "Checking credentials...".into());
        let is_authenticated = self.get_caller_identity().await.is_ok();

        if !is_authenticated {
            auth_spinner.stop_and_persist("🔐", "Authentication needed!".into());

            let mut s = Spinner::new(Spinners::Dots, "Performing SSO Authentication".into());

            self.login().await?;

            s.stop_and_persist("✅", "Authenticated!".into());
        } else {
            auth_spinner.stop_and_persist("✅", "Authenticated!".into());
        }

        Ok(())
    }
}

/// AWS API Methods
impl AWS {
    pub async fn login(&self) -> Result<()> {
        spawn_command!(
            "aws",
            "sso",
            "login",
            "--color",
            "off",
            "--profile",
            &self.profile
        )?
        .wait()
        .await?;

        Ok(())
    }

    pub async fn get_caller_identity(&self) -> Result<CallerIdentity> {
        let output = command!(
            "aws",
            "sts",
            "get-caller-identity",
            "--output",
            "json",
            "--color",
            "off",
            "--profile",
            &self.profile
        )
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

    pub async fn get_region(&mut self) -> Result<String> {
        let stdout = command!(
            "aws",
            "configure",
            "get",
            "region",
            "--color",
            "off",
            "--profile",
            &self.profile
        )
        .output()
        .await?
        .stdout;

        let region = String::from_utf8(stdout)?;
        let region = region.trim().to_string();

        self.region = Some(region.clone());

        Ok(region)
    }

    pub async fn get_pull_request(&self, id: String) -> Result<PullRequestResponse> {
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
            &self.profile
        )
        .output()
        .await?
        .stdout;

        let raw_stdout = String::from_utf8(stdout)?;

        Ok(serde_json::from_str(&raw_stdout)?)
    }

    pub async fn list_pull_requests(
        &self,
        repository: String,
        pull_request_status: PullRequestStatus,
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
            &self.profile
        )
        .output()
        .await?
        .stdout;

        let raw_output = String::from_utf8(stdout)?;

        Ok(serde_json::from_str(&raw_output)?)
    }

    pub async fn list_my_pull_requests(
        &self,
        repository: String,
        pull_request_status: PullRequestStatus,
        author_arn: String,
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
            "--author-arn",
            &author_arn,
            "--color",
            "off",
            "--profile",
            &self.profile
        )
        .output()
        .await?
        .stdout;

        let raw_output = String::from_utf8(stdout)?;

        Ok(serde_json::from_str(&raw_output)?)
    }

    pub async fn create_pull_request(
        &self,
        repository: String,
        title: String,
        description: String,
        source_branch: String,
        target_branch: String,
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
            &self.profile
        )
        .output()
        .await?
        .stdout;

        let string_output = String::from_utf8(stdout)?;

        Ok(serde_json::from_str(&string_output)?)
    }

    pub async fn merge_pr_by_squash(
        &self,
        id: String,
        repository: String,
        message: String,
        name: String,
        email: String,
    ) -> Result<PullRequest> {
        let stdout = command!(
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
            &self.profile,
            "--color",
            "off",
            "--output",
            "json"
        )
        .output()
        .await?
        .stdout;
        let raw_stdout = String::from_utf8(stdout)?;
        let json: PullRequestResponse = serde_json::from_str(&raw_stdout)?;

        Ok(json.pull_request)
    }
}
