use crate::command;
use color_eyre::Result;

#[derive(Clone)]
pub struct Branch(pub String);

impl ToString for Branch {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Branch {
    pub fn is_feature(&self) -> bool {
        self.0.starts_with("feature/")
    }

    pub fn is_hotfix(&self) -> bool {
        self.0.starts_with("hotfix/")
    }

    pub fn is_release(&self) -> bool {
        self.0.starts_with("release/")
    }
}

pub async fn force_push_to_env(remote: &str, env: &str) -> Result<()> {
    let branch = command!("git", "rev-parse", "--abbrev-ref", "HEAD")
        .output()
        .await?
        .stdout;
    let branch = String::from_utf8(branch)?;
    let branch = branch.trim();

    let target = format!("{branch}:{env}");

    command!("git", "push", "--force", remote, &target)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

pub async fn push(remote: &str, branch: Option<&str>) -> Result<()> {
    if let Some(branch) = branch {
        command!("git", "push", remote, branch).output().await?;
        return Ok(());
    }

    command!("git", "push", remote).output().await?;
    Ok(())
}

pub async fn push_tags() -> Result<()> {
    command!("git", "push", "--tags").output().await?;
    Ok(())
}

#[deprecated = "use `current_branch_v2` instead"]
pub async fn current_branch() -> Result<String> {
    let bytes = command!("git", "branch", "--show-current")
        .output()
        .await?
        .stdout;

    let branch = String::from_utf8(bytes)?;
    let branch = branch.trim();

    Ok(branch.to_string())
}

pub async fn current_branch_v2() -> Result<Branch> {
    let bytes = command!("git", "branch", "--show-current")
        .output()
        .await?
        .stdout;

    let branch = String::from_utf8(bytes)?;
    let branch = branch.trim();

    Ok(Branch(branch.to_string()))
}

pub async fn get_remote_url(remote: &str) -> Result<String> {
    let bytes = command!("git", "remote", "get-url", remote)
        .output()
        .await?
        .stdout;
    let branch = String::from_utf8(bytes)?;
    let branch = branch.trim();

    Ok(branch.to_string())
}

pub async fn delete_remote_branch(remote: &str, branch: String) -> Result<()> {
    let branch = format!(":{branch}");

    command!("git", "push", remote, &branch).output().await?;

    Ok(())
}

pub async fn config(key: String) -> Result<String> {
    let stdout = command!("git", "config", &key).output().await?.stdout;
    let out = String::from_utf8(stdout)?;
    let out = out.trim();

    Ok(out.to_string())
}

pub async fn fetch(prune: bool) -> Result<()> {
    if prune {
        command!("git", "fetch", "--prune").output().await?;
    } else {
        command!("git", "fetch").output().await?;
    }

    Ok(())
}

pub mod flow {
    pub mod release {
        use crate::command;

        use color_eyre::Result;

        pub async fn start(name: &str) -> Result<String> {
            let stdout = command!("git", "flow", "release", "start", name)
                .output()
                .await?
                .stdout;

            Ok(String::from_utf8(stdout)?.trim().to_string())
        }

        pub async fn finish(name: &str) -> Result<String> {
            let stdout = command!("git", "flow", "release", "finish", name)
                .output()
                .await?
                .stdout;

            Ok(String::from_utf8(stdout)?.trim().to_string())
        }
    }

    pub mod feature {
        use crate::command;

        use color_eyre::Result;

        pub async fn start(name: &str) -> Result<String> {
            let stdout = command!("git", "flow", "feature", "start", name)
                .output()
                .await?
                .stdout;

            Ok(String::from_utf8(stdout)?.trim().to_string())
        }

        pub async fn finish(name: &str) -> Result<String> {
            let stdout = command!("git", "flow", "feature", "finish", name)
                .output()
                .await?
                .stdout;

            Ok(String::from_utf8(stdout)?.trim().to_string())
        }
    }
}
