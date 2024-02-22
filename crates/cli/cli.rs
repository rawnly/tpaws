use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(long, short)]
    pub title: Option<String>,

    #[arg(long, short)]
    pub description: Option<String>,

    #[arg(long, short, default_value = "develop")]
    pub base: String,

    #[arg(long, default_value = "default")]
    pub profile: String,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub no_slack: bool,
}
