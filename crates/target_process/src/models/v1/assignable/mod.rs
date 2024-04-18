use serde::{Deserialize, Serialize};

mod assignables;
mod models;

pub use assignables::*;
pub use models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ID {
    pub id: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Project {
    pub id: usize,
    pub name: String,
    pub resource_type: String,
}
