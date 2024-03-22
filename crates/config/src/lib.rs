use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};
use std::env::var as env_var;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

pub(crate) fn dir() -> Option<PathBuf> {
    let user_directories = directories::UserDirs::new()?;
    let home = user_directories.home_dir().to_str()?;

    Some(
        env_var("XDG_CONFIG_HOME")
            .ok()
            .map(|s| format!("{s}/satispay.json"))
            .unwrap_or_else(|| format!("{}/.config/satispay.json", home))
            .into(),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub pr_name: String,
    pub pr_email: String,
    pub user_id: usize,
    pub last_auth: Option<DateTime<Utc>>,
    pub arn: Option<String>,
}

impl Config {
    pub fn update_auth(&mut self, arn: String) {
        self.last_auth = Some(Utc::now());
        self.arn = Some(arn);
    }

    pub fn is_auth_expired(&self) -> bool {
        let now = Utc::now();

        if let Some(last_auth) = self.last_auth {
            return now.signed_duration_since(last_auth).num_seconds() > 60 * 60 * 8;
        }

        true
    }

    pub fn is_first_run() -> Result<bool> {
        let path = dir().ok_or_eyre("unable to get config_dir")?;

        Ok(!path.exists())
    }

    pub fn write(&self) -> Result<()>
    where
        Self: Sized + serde::de::DeserializeOwned + serde::Serialize,
    {
        let path = dir().ok_or_eyre("unable to get config_dir")?;
        let file = std::fs::File::create(path.clone())?;

        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    pub async fn read() -> Result<Self>
    where
        Self: Sized + serde::de::DeserializeOwned + serde::Serialize,
    {
        let path = dir().ok_or_eyre("unable to get config_dir")?;

        if !path.exists() {
            return Err(eyre!("invalid config dir path"));
        }

        let mut file = tokio::fs::File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        Ok(serde_json::from_str(&contents)?)
    }
}
