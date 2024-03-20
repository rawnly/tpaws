use color_eyre::Result;
use commands::git;
use config::Config;
use inquire::Text;

pub async fn reset() -> Result<()> {
    let username = Text::new("Satispay Username:")
        .with_help_message("e.g: name.surname")
        .prompt()?;

    let name = git::config("user.name".to_string()).await?;
    let email = git::config("user.email".to_string()).await?;

    let pr_name = Text::new("Your full name:").prompt_skippable()?;

    let pr_email = Text::new("Your email:")
        .with_help_message("default to your git config user.email")
        .with_default(&format!("{username}@satispay.com"))
        .prompt_skippable()?;

    let me = target_process::get_me().await?;

    let config = Config {
        username,
        pr_name: pr_name.unwrap_or(name),
        pr_email: pr_email.unwrap_or(email),
        user_id: me.id,
    };

    config.write()
}
