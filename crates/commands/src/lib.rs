use std::process::{Command, Stdio};

pub mod aws;
pub mod git;

pub fn has(command: &str) -> bool {
    Command::new(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}
