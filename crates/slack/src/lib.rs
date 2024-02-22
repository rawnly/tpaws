use color_eyre::Result;

mod http;
pub mod models;

use crate::models::*;

pub async fn send_message(
    content: String,
    pr_link: String,
    tp_link: String,
) -> Result<reqwest::Response> {
    let client = http::make_client()?;

    let url = "https://hooks.slack.com/services/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";

    let payload = Message::default().blocks(vec![
        Block::section(BlockType::Mrkdwn, &content),
        Block::divider(),
        Block::actions(vec![
            Button::link(&pr_link, "Code Commit"),
            Button::link(&tp_link, "Target Process"),
        ]),
    ]);

    let response = client.post(url).json(&payload).send().await?;

    Ok(response)
}
