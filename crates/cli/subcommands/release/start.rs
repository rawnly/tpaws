use color_eyre::{eyre::eyre, Result};
use commands::{command, git};
use global_utils::print_dbg;
use std::str::FromStr;

use crate::manifests::{node::PackageJson, Version};

#[derive(strum::Display, Debug, Clone, Copy)]
pub enum ReleaseKind {
    Patch,
    Minor,
    Major,
}

pub async fn start(pkg: &PackageJson, release_kind: ReleaseKind) -> Result<()> {
    let mut pkg = pkg.clone();
    let prev_version = pkg.version.clone();
    let mut version = Version::from_str(&pkg.version).map_err(|_| eyre!("invalid version"))?;

    print_dbg!(&version);

    // bump
    match release_kind {
        ReleaseKind::Patch => version.bump_patch(),
        ReleaseKind::Minor => version.bump_minor(),
        ReleaseKind::Major => version.bump_major(),
    };

    print_dbg!(&release_kind);

    pkg.version = version.to_string();

    git::flow::release::start(&version.to_string()).await?;

    let command = &release_kind.to_string();
    let command = command.to_lowercase();

    command!("npm", "version", &command).output().await?;

    println!("version bumped from {prev_version} to: {}", pkg.version);

    Ok(())
}
