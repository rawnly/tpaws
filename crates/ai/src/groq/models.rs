use serde::{Deserialize, Serialize};

pub const ENV_NAME: &str = "GROQ_API_KEY";

pub fn get_apikey_from_env() -> Option<String> {
    std::env::var(ENV_NAME).ok()
}

#[derive(Debug, Clone, Deserialize, Serialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Role {
    #[strum(serialize = "user")]
    User,

    #[strum(serialize = "system")]
    System,

    #[strum(serialize = "assistant")]
    Assistant,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }

    pub fn role(&self) -> Role {
        self.role.clone()
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }

    /// Create a new system message
    pub fn system(content: String) -> Self {
        Self::new(Role::System, content)
    }

    /// Create a new user message
    pub fn user(content: String) -> Self {
        Self::new(Role::User, content)
    }

    /// Create a new assistant message
    pub fn assisstant(content: String) -> Self {
        Self::new(Role::Assistant, content)
    }
}
