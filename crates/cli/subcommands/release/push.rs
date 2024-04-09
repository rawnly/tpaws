use color_eyre::{eyre::Context, Result};
use commands::git;

use crate::cli::ReleasePushTarget;

pub async fn push(target: ReleasePushTarget) -> Result<()> {
    match target {
        ReleasePushTarget::Prod => {
            git::force_push_to_env("origin", "prod")
                .await
                .context("failed prod push")?;
        }
        ReleasePushTarget::Staging => {
            git::force_push_to_env("origin", "staging")
                .await
                .context("failed staging push")?;
        }
        ReleasePushTarget::All => {
            git::force_push_to_env("origin", "staging")
                .await
                .context("failed to push to staging")?;

            git::force_push_to_env("origin", "prod")
                .await
                .context("failed to push to prod")?;
        }
    }

    Ok(())
}
