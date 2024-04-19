use regex::Regex;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;

#[derive(Debug)]
enum Profile {
    Debug,
    Release,
    Test,
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = match s {
            "debug" => Profile::Debug,
            "release" => Profile::Release,
            "test" => Profile::Test,
            _ => return Err(format!("unable to convert profile: \"{s}\"")),
        };

        Ok(p)
    }
}

fn get_profile() -> Profile {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let re = Regex::new(r#"target/(release|test|debug)"#).unwrap();

    let captures = re.captures(&out_dir);

    let profile_str = captures
        .unwrap()
        .get(1)
        .map(|a| a.as_str().to_string())
        .ok_or("cannot extract profile")
        .unwrap();

    Profile::from_str(&profile_str).unwrap()
}

fn last_commit() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap()
        .stdout;

    String::from_utf8(output).unwrap()
}

fn main() -> io::Result<()> {
    let profile = get_profile();
    let commit = last_commit();

    std::env::set_var("TPAWS_COMMIT_ID", commit);

    if matches!(profile, Profile::Release) {
        println!("cargo:warning=Building formula");

        generate_formula(
            "tpaws",
            "https://github.com/rawnly/tpaws",
            "CLI to manage TargetProcess and AWS CodeCommit",
        )?;
    }

    Ok(())
}

fn generate_formula(bin_name: &str, repo: &str, description: &str) -> io::Result<()> {
    let string_template = fs::read_to_string("./formula_template.rb")?;
    let formula = string_template
        .replace("{{bin}}", bin_name)
        .replace("{{description}}", description)
        .replace("{{homepage}}", repo)
        .replace("{{repo}}", repo)
        // .replace("{{shasum}}", &shasum)
        .replace("{{version}}", env!("CARGO_PKG_VERSION"));

    let mut file = File::create(format!("{bin_name}.rb"))?;
    write!(file, "{formula}")?;

    Ok(())
}
