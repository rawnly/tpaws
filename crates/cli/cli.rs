use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum TicketCommands {
    Get {
        id_or_url: String,

        #[arg(long)]
        json: bool,
    },
    Start {
        id_or_url: String,

        #[arg(long, short)]
        branch: Option<String>,

        #[arg(long)]
        no_git: bool,

        #[arg(long)]
        no_assign: bool,
    },
    Open {
        id_or_url: Option<String>,
    },
    Link {
        id_or_url: Option<String>,
    },
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum Commands {
    Ticket {
        #[command(subcommand)]
        subcommands: TicketCommands,
    },
    CreatePR {
        #[arg(long)]
        no_slack: bool,

        #[arg(long, short)]
        title: Option<String>,

        #[arg(long, short)]
        description: Option<String>,

        #[arg(long, short, default_value = "develop")]
        base: String,

        #[arg(long, default_value = "default")]
        profile: String,
    },
}
