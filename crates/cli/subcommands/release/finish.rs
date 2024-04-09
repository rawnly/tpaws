use color_eyre::{eyre::Context, Result};
use commands::git;

use crate::manifests::node::PackageJson;

pub async fn finish(pkg: &PackageJson, branch: &String) -> Result<()> {
    git::flow::release::finish(&pkg.version)
        .await
        .context("failed to finish release")?;

    git::push("origin", Some("master"))
        .await
        .context("failed to push master")?;

    git::push("origin", Some("develop"))
        .await
        .context("failed to push develop")?;

    git::push_tags().await.context("failed to push tags")?;

    git::delete_remote_branch("origin", branch.to_string())
        .await
        .context("failed to delete remote branch")?;

    Ok(())
}
