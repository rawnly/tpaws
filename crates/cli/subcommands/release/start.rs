use color_eyre::{eyre::eyre, Result};
use commands::{command, git};
use std::str::FromStr;

use crate::manifests::{node::PackageJson, Version};

pub async fn start(pkg: &PackageJson) -> Result<()> {
    let mut pkg = pkg.clone();
    let prev_version = pkg.version.clone();
    let mut version = Version::from_str(&pkg.version).map_err(|_| eyre!("invalid version"))?;

    // bump
    version.bump_minor();
    pkg.version = version.to_string();

    git::flow::release::start(&version.to_string()).await?;
    command!("npm", "version", "minor").output().await?;

    println!("version bumped from {prev_version} to: {}", pkg.version);

    Ok(())
}
