use std::path::Path;

use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

pub trait File {
    fn read(path: &Path) -> Result<Self>
    where
        Self: Sized + serde::de::DeserializeOwned + serde::Serialize;

    fn write(&self, path: &Path) -> Result<()>
    where
        Self: Sized + serde::de::DeserializeOwned + serde::Serialize;
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {}

impl Config {
    async fn write(&self, path: &Path) -> Result<()>
    where
        Self: Sized + serde::de::DeserializeOwned + serde::Serialize,
    {
        let file = tokio::fs::File::open(path).await?;

        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    async fn read(path: &Path) -> Result<Self>
    where
        Self: Sized + serde::de::DeserializeOwned + serde::Serialize,
    {
        if !path.exists() {
            return Err(eyre!(""));
        }

        let mut file = tokio::fs::File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        Ok(serde_json::from_str(&contents)?)
    }
}
