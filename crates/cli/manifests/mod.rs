use regex::*;
use serde::Deserialize;
use std::str::FromStr;

pub mod node;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl Version {
    pub fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

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
        format!("{}.{:0>1}.{:0>1}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split('.').collect();

        Ok(Self {
            major: data.first().unwrap().parse().unwrap_or(0),
            minor: data.get(1).unwrap().parse().unwrap_or(0),
            patch: data.last().unwrap_or(&"0").parse().unwrap_or(0),
        })
    }
}

impl Version {
    pub fn is_valid(version: &str) -> bool {
        let re = Regex::new(r#"\d{1,2}\.\d+(\.\d+)?"#).unwrap();

        re.is_match(version)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::manifests::Version;

    #[test]
    fn check_version() {
        let versions = vec![("1.0.1", true), ("1.100.24", true), ("1.0", true)];

        for (k, v) in versions {
            assert_eq!(Version::is_valid(k), v);
        }
    }

    #[test]
    fn extract_test() {
        let versions = vec![
            ("1.0.1", Version::new(1, 0, 1)),
            ("1.00.0", Version::new(1, 0, 0)),
            ("1.100.1", Version::new(1, 100, 1)),
            ("2.53.1", Version::new(2, 53, 1)),
            ("1.00.01", Version::new(1, 0, 1)),
        ];

        for (k, v) in versions {
            assert_eq!(Version::from_str(k).unwrap(), v);
        }
    }

    #[test]
    fn to_string() {
        let versions = vec![
            ("1.00.01", Version::new(1, 0, 1)),
            ("1.00.01", Version::new(1, 0, 1)),
            ("1.00.00", Version::new(1, 0, 0)),
            ("1.100.01", Version::new(1, 100, 1)),
            ("2.53.01", Version::new(2, 53, 1)),
            ("1.00.01", Version::new(1, 0, 1)),
        ];

        for (k, v) in versions {
            assert_eq!(v.to_string(), k);
        }
    }
}
