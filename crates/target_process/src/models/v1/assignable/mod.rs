use serde::{Deserialize, Serialize};

mod assignable;
mod models;

pub use assignable::*;
pub use models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ID {
    pub id: usize,
}
