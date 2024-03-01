use crate::command;
use color_eyre::Result;

pub async fn current_branch() -> Result<String> {
    let bytes = command!("git", "branch", "--show-current")
        .output()
        .await?
        .stdout;

    let branch = String::from_utf8(bytes)?;
    let branch = branch.trim();

    Ok(branch.to_string())
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

pub async fn fetch(prune: bool) -> Result<()> {
    if prune {
        command!("git", "fetch", "--prune").output().await?;
    } else {
        command!("git", "fetch").output().await?;
    }

    Ok(())
}

pub mod flow {
    pub mod feature {
        use crate::spawn_command;

        use color_eyre::Result;

        pub async fn start(name: &str) -> Result<()> {
            spawn_command!("git", "flow", "feature", "start", name)?;

            Ok(())
        }

        pub async fn finish(name: &str) -> Result<()> {
            spawn_command!("git", "flow", "feature", "finish", name)?;
            Ok(())
        }
    }
}
