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

impl Assignable {
    pub fn get_link(self) -> String {
        format!("https://satispay.tpondemand.com/entity/{id}", id = self.id).to_string()
    }

    pub fn get_branch(self) -> String {
        let mut name = self.name.clone().to_lowercase();
        name.retain(|x| {
            ![
                '(', ')', '[', ']', '{', '}', ',', '\"', '/', '.', ';', ':', '\'', '-', '_',
            ]
            .contains(&x)
        });

        format!("{}_{}", self.id, name.replace(' ', "_"))
    }
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ID {
    pub id: usize,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateEntityStatePayload {
    pub id: usize,
    pub entity_state: ID,
}
