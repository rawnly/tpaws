use super::ID;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GeneralUser {
    pub id: usize,
    pub first_name: String,
    pub last_name: String,
    pub login: String,
    pub full_name: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdAndName {
    pub id: usize,
    pub name: String,
}

pub type EntityState = IdAndName;
pub type EntityType = IdAndName;

// PAYLOAD

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateEntityStatePayload {
    pub id: usize,
    pub entity_state: ID,
}
