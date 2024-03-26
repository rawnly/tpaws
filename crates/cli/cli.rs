use clap::{Parser, Subcommand};
use commands::aws::PullRequestStatus;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// do not perform any action
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// do not print any output/spinner
    #[arg(long, global = true)]
    pub quiet: bool,
}

#[derive(Debug, Clone, strum::Display, strum::EnumString)]
pub enum ReleasePushTarget {
    Staging,
    Prod,
    All,
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum ReleaseCommands {
    Start,
    Push {
        #[arg(long, short, default_value = "All")]
        target: ReleasePushTarget,
    },
    Finish,
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum TicketCommands {
    /// Info about the current branch or given userStory id
    View {
        /// userStory ID or URL
        id_or_url: Option<String>,

        /// Opens userStory in the browser
        #[arg(long, short)]
        web: bool,

        /// Opens userStory in the browser
        #[arg(long)]
        json: bool,
    },
    /// Run `git flow start` and update status/assigned developer
    Start {
        /// userStory ID or URL
        id_or_url: Option<String>,

        /// Branch name (by default it's autogenerated from userStory title)
        #[arg(long, short)]
        branch: Option<String>,

        /// Do not perform git flow actions
        #[arg(long)]
        no_git: bool,

        /// Do not update userStory status/developer
        #[arg(long)]
        no_assign: bool,
    },
    /// Run `git flow finish`
    Finish {
        id_or_url: Option<String>,
    },

    /// Print userStory link
    Link {
        id_or_url: Option<String>,
    },

    GetId {
        url: Option<String>,
    },
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum PullRequestCommands {
    /// Craete a PR
    Create {
        /// Do not notify slack channel
        #[arg(long)]
        slack: bool,

        /// PR title
        #[arg(long, short)]
        title: Option<String>,

        /// PR description
        #[arg(long, short)]
        description: Option<String>,

        /// PR base branch
        #[arg(long, short, default_value = "develop")]
        base: String,
    },
    /// Retrive a PR
    View {
        /// PR id
        id: Option<String>,

        /// open the PR in the browser
        #[arg(long, short)]
        web: bool,
    },
    /// Squash merge a PR
    Merge {
        /// do not prompt for confirmation
        #[arg(long, short)]
        use_defaults: bool,

        #[arg(long, short)]
        commit_message: Option<String>,

        #[arg(long)]
        author: Option<String>,

        #[arg(long)]
        email: Option<String>,

        id: Option<String>,
    },

    List {
        #[arg(long)]
        interactive: bool,

        #[arg(long, short)]
        status: Option<PullRequestStatus>,
    },
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum ConfigCommands {
    Reset,
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum Commands {
    /// Manage configs
    #[clap(visible_alias = "cfg")]
    Config {
        #[command(subcommand)]
        subcommands: ConfigCommands,
    },

    /// Release
    Release {
        #[command(subcommand)]
        subcommands: ReleaseCommands,
    },

    /// Manage target process
    #[clap(visible_alias = "us")]
    Ticket {
        #[command(subcommand)]
        subcommands: TicketCommands,
    },
    /// Craete / Manage and List pull requests
    #[clap(visible_alias = "pr")]
    PullRequest {
        #[command(subcommand)]
        subcommands: PullRequestCommands,

        /// aws profile
        #[arg(long, default_value = "default")]
        profile: String,
    },
}
