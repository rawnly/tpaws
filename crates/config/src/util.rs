use std::path::Path;

use color_eyre::{eyre::eyre, Result};
use tokio::fs::{read_to_string, write};

#[derive(Debug)]
pub enum Shell {
    Bash,
    Zsh,
}

impl TryFrom<String> for Shell {
    type Error = color_eyre::Report;

    fn try_from(value: String) -> std::prelude::v1::Result<Self, Self::Error> {
        match value.as_ref() {
            "/bin/bash" => Ok(Self::Bash),
            "/bin/zsh" => Ok(Self::Zsh),
            _ => Err(eyre!("unsupported shell")),
        }
    }
}

impl ToString for Shell {
    fn to_string(&self) -> String {
        match self {
            Self::Bash => ".bashrc",
            Self::Zsh => ".zshrc",
        }
        .into()
    }
}

pub fn get_user_id() -> Option<String> {
    let username = std::env::var("USER").ok()?;
    let hostname = std::env::var("HOST").ok()?;

    Some(sha256::digest(format!("{}:{}", username, hostname)))
}

// we don't return error if the write operation fails
// instead show a warning message to the user
pub async fn inject_env(key: &str, value: &str) -> Result<()> {
    std::env::set_var(key, value);

    if let (Ok(shell), Ok(home)) = (
        Shell::try_from(std::env::var("SHELL")?),
        std::env::var("HOME"),
    ) {
        let home = Path::new(&home);
        let rcfile = home.join(shell.to_string());

        if let Ok(mut content) = read_to_string(&rcfile).await {
            content.push_str(&format!("\nexport {key}={value}"));

            if write(&rcfile, content).await.is_ok() {
                return Ok(());
            };
        }
    }

    println!(
        r#"
WARNING!

Failed to update your shell environment.
Please update your rc file (`.zshrc` or whatever) with the following configuration:

```
export {key}={value}
```

            "#
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use std::env::temp_dir;
    use tokio::fs;

    use crate::util::get_user_id;

    /// Test variable injection with `ZSH` shell
    #[tokio::test]
    async fn test_inject_env_zsh() {
        let home = temp_dir();
        let rcfile = home.clone().join(".zshrc");
        println!("{:?}", rcfile);

        fs::write(&rcfile, "A=B").await.unwrap();

        std::env::set_var("SHELL", "/bin/zsh");
        std::env::set_var("HOME", home.to_str().unwrap());

        super::inject_env("foo", "bar").await.unwrap();
    }

    #[test]
    fn test_user_id() {
        let id = get_user_id();
        let id2 = get_user_id();

        assert_eq!(id, id2)
    }
}
