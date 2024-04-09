use super::models::*;
use crate::{get_base_url, models::v2::assignable::Assignable as AssignableV2};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Assignable {
    /// UserStory - Bug
    pub resource_type: String,
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub entity_state: EntityState,
    pub entity_type: EntityType,
}

impl Assignable {
    pub fn get_link(self) -> String {
        let target_process_url = get_base_url();

        format!("{target_process_url}/entity/{id}", id = self.id).to_string()
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

    pub fn is_bug(&self) -> bool {
        self.entity_type.name.to_lowercase() == "bug"
    }

    pub fn is_user_story(&self) -> bool {
        self.entity_type.name.to_lowercase() == "userstory"
    }
}

impl From<AssignableV2> for Assignable {
    fn from(
        AssignableV2 {
            id,
            resource_type,
            name,
            description,
            entity_type,
            entity_state,
        }: AssignableV2,
    ) -> Self {
        Self {
            id,
            name,
            description,
            resource_type,
            entity_state: entity_state.into(),
            entity_type: entity_type.into(),
        }
    }
}
