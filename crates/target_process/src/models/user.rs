use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CurrentUser {
    pub id: usize,
    pub first_name: String,
    pub last_name: String,
    pub login: String,
    pub email: String,
    pub is_active: bool,
    pub role: Role,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Role {
    pub id: usize,
    pub name: String,
}
