use color_eyre::Result;
use std::path::Path;
use tokio::io::AsyncReadExt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project_id: Option<String>,
    pub tp_name: Option<String>,
}

const FILE_PATH: &str = "./tpaws.json";

/// FS Methods
impl ProjectConfig {
    /// Read from file-system
    pub fn write(&self) -> Result<()>
    where
        Self: Sized + serde::de::DeserializeOwned + serde::Serialize,
    {
        let path = Path::new(FILE_PATH);
        let file = std::fs::File::create(path)?;

        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    /// Write to file-system
    pub async fn read() -> Option<Self>
    where
        Self: Sized + serde::de::DeserializeOwned + serde::Serialize,
    {
        let path = Path::new(FILE_PATH);

        if !path.exists() {
            return None;
        }

        let mut file = tokio::fs::File::open(path).await.ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await.ok()?;

        serde_json::from_str(&contents).ok()
    }
}
