#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Assignable {
    pub resource_type: String,
    pub id: usize,
    pub name: String,
    pub description: String,
    pub last_editor: GeneralUser,
    pub owner: GeneralUser,
    pub creator: GeneralUser,
    pub entity_state: EntityState,
    pub priority: Priority,
    pub team: Team,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GeneralUser {
    pub id: usize,
    pub first_name: String,
    pub last_name: String,
    pub login: String,
    pub full_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Team {
    pub id: usize,
    pub name: String,
    pub emoji_icon: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Priority {
    pub id: usize,
    pub name: String,
    pub importance: usize,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EntityState {
    pub id: usize,
    pub name: String,
}
