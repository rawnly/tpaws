use color_eyre::Result;
use commands::git;
use config::Config;
use inquire::Text;

pub async fn reset() -> Result<()> {
    let me = target_process::get_me().await?;
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

    let username = Text::new("Satispay Username:")
        .with_help_message("e.g: name.surname")
        .with_default(&potential_username.unwrap_or(me.login))
        .prompt()?;

    let config = Config {
        username,
        pr_name: pr_name.unwrap_or(name),
        pr_email: pr_email.unwrap_or(email),
        user_id: me.id,
        last_auth: None,
        arn: None,
    };

    config.write()
}
