use serde::Deserialize;
use std::str::FromStr;

pub mod node;

#[derive(Debug, Clone, Deserialize)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl Version {
    /// Bumps minor and sets patch to 0
    pub fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    pub fn bump_patch(&mut self) {
        self.patch += 1
    }

    pub fn bump_major(&mut self) {
        self.patch = 0;
        self.minor = 0;
        self.major += 1;
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split('.').collect();

        if data.len() < 3 {
            return Err(());
        }

        Ok(Self {
            major: data.first().unwrap().parse().unwrap(),
            minor: data.get(1).unwrap().parse().unwrap(),
            patch: data.last().unwrap().parse().unwrap(),
        })
    }
}
