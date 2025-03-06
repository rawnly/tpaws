use cached::proc_macro::cached;
use color_eyre::{eyre::OptionExt, Result};
use commands::git;
use config::Config;
use inquire::Text;
use regex::Regex;

#[cached]
pub(crate) fn build_pr_link(region: String, repository: String, id: String) -> String {
    format!("https://{region}.console.aws.amazon.com/codesuite/codecommit/repositories/{repository}/pull-requests/{id}/details")
}

pub(crate) async fn get_repository() -> Result<String> {
    let remote = git::get_remote_url("origin").await?;

    remote
        .split('/')
        .last()
        .map(|s| s.to_string())
        .ok_or_eyre("unable to extract repository from origin")
}

pub(crate) fn branch_to_title(branch: String) -> String {
    let re = Regex::new(r#"\d+"#).unwrap();

    let sanitized_branch_title = re
        .replace(branch.split('/').last().unwrap_or(&branch), "")
        .replace('_', " ")
        .trim()
        .to_string();

    let (first, rest) = sanitized_branch_title.split_at(1);

    // capitalize
    format!("{}{rest}", first.to_uppercase())
}

pub(crate) fn get_ticket_id_from_branch(branch: String) -> Option<String> {
    let re = Regex::new(r#"\w+\/(\d+)_.*"#).unwrap();
    let captures = re.captures(&branch)?;

    captures.get(1).map(|s| s.as_str().to_string())
}

pub(crate) async fn grab_title(title: Option<String>, branch: String) -> Result<String> {
    match title {
        Some(title) => Ok(title),
        None => {
            if !target_process::has_token() {
                return Ok(branch_to_title(branch));
            }

            let id =
                get_ticket_id_from_branch(branch).ok_or_eyre("failed to retrive user_story ID")?;

            let assignable = target_process::get_assignable(id).await?;

            Ok(assignable.name)
        }
    }
}

pub(crate) fn extract_id_from_url(url: String) -> Option<String> {
    let regex = Regex::new(r#"https?:\/\/\w+\.tpondemand\.com\/entity\/(\d+)([\w+-]+)"#).ok()?;

    if regex.is_match(&url) {
        if let Some(captures) = regex.captures(&url) {
            return captures.get(1).map(|s| s.as_str().to_string());
        }
    }

    None
}

pub(crate) async fn extract_id(id_or_url: Option<String>) -> Result<String> {
    let id = id_or_url.map(|id_or_url| extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url));

    match id {
        Some(id) => Ok(id),
        None => {
            let branch = git::current_branch_v2().await?.0;
            get_ticket_id_from_branch(branch).ok_or_eyre("Unable to extract userStory ID")
        }
    }
}

pub(crate) fn get_groq_api_key(config: &Config) -> Result<String> {
    config
        .clone()
        .groq_api_key
        .or_else(ai::groq::models::get_apikey_from_env)
        .ok_or_eyre("No api key. Please provide a groq api key in order to use this feature")
}

pub(crate) fn get_groq_api_key_or_prompt(config: &mut Config) -> Result<String> {
    match get_groq_api_key(config) {
        Ok(k) => Ok(k),
        Err(e) => match Text::new("Enter your Groq API key:")
            .with_placeholder("gsk_XX")
            .prompt_skippable()?
        {
            None => return Err(e),
            Some(k) => {
                config.update_groq_api_key(&k);
                config.write()?;
                Ok(k)
            }
        },
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn extract_id() {
        let url = "https://company.tpondemand.com/entity/125371-show-current-and-previous-month-in"
            .to_string();
        let result = super::extract_id_from_url(url.clone());

        assert!(result.is_some());
        assert_eq!(result, Some("125371".to_string()));
    }

    #[test]
    fn branch_to_title() {
        let string = "feature/115068_translate_report_type_payout_transactions".to_string();
        let output = super::branch_to_title(string);

        assert_eq!(
            output,
            "Translate report type payout transactions".to_string()
        )
    }

    #[test]
    fn extract_assignable_id() {
        let string = "feature/115068_translate_report_type_payout_transactions".to_string();

        let output = super::get_ticket_id_from_branch(string);

        assert_eq!(output, Some("115068".into()))
    }

    #[tokio::test]
    async fn grab_title_should_not_execute_async_code_if_title_is_given() {
        let data = super::grab_title(Some("demo".to_string()), "feature/120890_abc".to_string())
            .await
            .unwrap();

        assert_eq!("demo".to_string(), data);
    }
}
