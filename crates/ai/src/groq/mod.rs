pub mod models;

use color_eyre::eyre::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use models::Message;

#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
    api_key: String,
}

struct BearerAuth(String);

impl From<BearerAuth> for HeaderValue {
    fn from(value: BearerAuth) -> Self {
        HeaderValue::from_str(&format!("Bearer {}", &value.0)).unwrap()
    }
}

impl Client {
    pub fn new(api_key: &str) -> Client {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, "application/json".parse().unwrap());
                headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
                headers.insert(AUTHORIZATION, BearerAuth(api_key.to_string()).into());

                headers
            })
            .build()
            .unwrap();

        Client {
            client,
            api_key: api_key.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatPayload {
    model: String,
    messages: Vec<Message>,
}

impl ChatPayload {
    pub fn new(model: &str, messages: Vec<Message>) -> ChatPayload {
        ChatPayload {
            model: model.to_string(),
            messages,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
}

impl Client {
    pub async fn chat(self, payload: ChatPayload) -> Result<ChatResponse> {
        let response = self
            .client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header(AUTHORIZATION, BearerAuth(self.api_key))
            .json(&payload)
            .send()
            .await?;

        Ok(response.json::<ChatResponse>().await?)
    }
}
