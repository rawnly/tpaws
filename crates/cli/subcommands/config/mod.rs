use ai::groq;
use color_eyre::{eyre::eyre, Result};
use colored::*;
use commands::git;
use config::{util::inject_env, Config, DEFAULT_AI_MODEL};
use inquire::Text;

pub async fn reset() -> Result<()> {
    if !target_process::is_configured() {
        let prompt = "Invalid configuration detected. Do you want to fix this now?".to_string();

        let fix = inquire::Confirm::new(&prompt).with_default(true).prompt()?;

        if !fix {
            return Err(eyre!(
                "Please make sure to have the correct configuration before continuing.\nMissing env `{}`",
                target_process::ENV_NAME.yellow()
            ));
        }

        let tpurl = inquire::Text::new("Target Process base url:")
            .with_help_message("e.g https://my-company.tpondemand.com")
            .prompt()?;

        inject_env(target_process::ENV_NAME, &tpurl).await?;
    }

    let me = match target_process::get_me().await {
        Ok(me) => me,
        Err(e) => panic!("Unable to retrive current user: {:?}", e),
    };

    let name = git::config("user.name".to_string())
        .await
        .unwrap_or(format!("{} {}", me.first_name, me.last_name));

    let email = git::config("user.email".to_string())
        .await
        .unwrap_or(me.email);

    let pr_name = Text::new("Your full name:")
        .with_help_message("default to your git config user.name")
        .with_default(&name)
        .prompt_skippable()?;

    let pr_email = Text::new("Your email:")
        .with_help_message("default to your git config user.email")
        .with_default(&email)
        .prompt_skippable()?;

    let potential_username = pr_name.as_ref().map(|name| {
        let s: Vec<&str> = name.split(' ').collect();
        s.join(".").to_lowercase()
    });

    let username = Text::new("TP Username:")
        .with_help_message("e.g: name.surname")
        .with_default(&potential_username.unwrap_or(me.login))
        .prompt()?;

    let groq_api_key = Text::new("Groq API Key:")
        .with_help_message("Get your API key from https://groq.dev")
        .with_default(&groq::models::get_apikey_from_env().unwrap_or_default())
        .prompt_skippable()?;

    let ai_model = Text::new("AI Model:")
        .with_help_message("https://console.groq.com/docs/models#production-models")
        .with_default(DEFAULT_AI_MODEL)
        .prompt_skippable()?;

    let tp_url = Text::new("Target Process URL")
        .with_placeholder("https://my-company.tpondemand.com")
        .with_default(&target_process::get_base_url())
        .prompt_skippable()?;

    let tp_apikey = Text::new("Target Process URL")
        .with_default(&target_process::get_token().ok().unwrap_or_default())
        .prompt_skippable()?;

    let config = Config {
        username,
        pr_name: pr_name.unwrap_or(name),
        pr_email: pr_email.unwrap_or(email),
        user_id: me.id,
        last_auth: None,
        arn: None,
        groq_api_key,
        ai_model,
        tp_url,
        tp_apikey,
    };

    config.write()
}
