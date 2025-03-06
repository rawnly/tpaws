use crate::utils;
use ai::groq;
use color_eyre::{eyre::OptionExt, Result};
use config::Config;
use inquire::Text;
use target_process::models::assignable::Assignable;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CommitMessage {
    message: String,
    description: Option<String>,
}

pub async fn generate_commit(
    id_or_url: Option<String>,
    json: bool,
    title_only: bool,
    config: &mut Config,
) -> Result<()> {
    let groq_api_key = match utils::get_groq_api_key(config) {
        Ok(k) => k,
        Err(e) => match Text::new("Enter your Groq API key:")
            .with_placeholder("gsk_XX")
            .prompt_skippable()?
        {
            None => return Err(e),
            Some(k) => {
                config.update_groq_api_key(&k);
                config.write()?;
                k
            }
        },
    };

    let id = utils::extract_id(id_or_url).await?;

    let assignable = target_process::get_assignable(id).await?;
    let messages = get_messages(assignable);
    let payload = groq::ChatPayload::new("llama3-8b-8192", messages);

    let ai = groq::Client::new(&groq_api_key);
    let response = ai.chat(payload).await?;
    let content = &response
        .choices
        .first()
        .ok_or_eyre("Invalid AI response. No choices returned.")?
        .message
        .content;

    let commit = serde_json::from_str::<CommitMessage>(content)?;

    if json {
        let str = serde_json::to_string_pretty(&commit)?;
        println!("{}", str);

        return Ok(());
    }

    println!("{}", commit.message);

    if !title_only {
        if let Some(description) = commit.description {
            println!("{}", description);
        }
    }

    Ok(())
}

fn get_messages(assignable: Assignable) -> Vec<groq::models::Message> {
    let system_prompt = groq::models::Message::system(
        r#"
                You're an ai specialist, you should know how to generate a commit message.
                Given ID, title and content of a User Story generate a commit message for it.

                The format should follow conventional commits.
                    - "feat(<id>): <message>" -> user stories
                    - "fix(<id>): <message>" -> bugs

                Return the commit message as response. Just that, no other information is needed.
                On a new line, add a longer description for the commit message if needed.

                The commit message must be lowercased and in present tense.
                Don't just copy the title, but provide a meaningful message.

                Return a JSON object with the following structure:
                {
                    "message": "feat(123): add new feature",
                    "description": "This feature will allow users to do X and Y" // optional
                }
                "#
        .to_string(),
    );

    let Assignable {
        id,
        name,
        description,
        entity_type,
        ..
    } = assignable;

    let message = groq::models::Message::user(format!(
        r#"
                    ID: {id}
                    Title: {name}
                    Type: {}
                    Description:
                    {}
                    "#,
        entity_type.name,
        &description
            .clone()
            .get_or_insert("No description provided".to_string())
    ));

    vec![system_prompt, message]
}
