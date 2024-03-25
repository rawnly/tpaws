use color_eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    pub name: String,
    pub version: String,

    #[serde(flatten)]
    pub other: serde_json::Value,
}

impl PackageJson {
    pub async fn exists() -> bool {
        tokio::fs::try_exists(Self::file_name()).await.is_ok()
    }

    pub async fn read() -> Result<Self> {
        let str = tokio::fs::read_to_string(Self::file_name()).await?;
        Ok(serde_json::from_str(&str)?)
    }

    pub fn file_name() -> String {
        "package.json".into()
    }
}
