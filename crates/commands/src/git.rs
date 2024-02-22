use color_eyre::Result;
use tokio::process::Command;

macro_rules! git {
    ($( $x:expr ),* ) => {
        #[allow(clippy::vec_init_then_push)]
        {
            let mut args = Vec::new();

            $(
                args.push($x);
            )*

            if cfg!(debug_assertions) {
                dbg!(&args);
            }

            Command::new("git")
                .args(&args.clone())
        }
    };
}

pub async fn current_branch() -> Result<String> {
    let bytes = git!("branch", "--show-current").output().await?.stdout;
    let branch = String::from_utf8(bytes)?;

    Ok(branch)
}

pub async fn get_remote_url(remote: &str) -> Result<String> {
    let bytes = git!("remote", "get-url", remote).output().await?.stdout;
    let branch = String::from_utf8(bytes)?;

    Ok(branch)
}

pub mod flow {
    pub mod feature {}
    pub mod hotfix {}
    pub mod release {}
}
