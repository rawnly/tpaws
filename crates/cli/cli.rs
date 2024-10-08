use clap::{Parser, Subcommand};
use commands::aws::PullRequestStatus;

#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
// #[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// do not perform any action
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// do not print any output/spinner
    #[arg(long, global = true)]
    pub quiet: bool,

    /// do not print any output/spinner
    #[arg(long, global = true)]
    pub debug: bool,

    /// print current version
    #[arg(long, short = 'v')]
    pub version: bool,
}

#[derive(Debug, Clone, strum::Display, strum::EnumString)]
pub enum ReleasePushTarget {
    Staging,
    Prod,
    All,
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum ReleaseCommands {
    Start {
        #[arg(long)]
        patch: bool,

        #[arg(long, default_value = "true")]
        minor: bool,

        #[arg(long)]
        major: bool,
    },
    Push {
        #[arg(long, short, default_value = "All")]
        target: ReleasePushTarget,

        /// pipeline to trigger
        #[arg(long, short = 'n')]
        pipeline_name: Option<String>,

        /// aws profile
        #[arg(long, default_value = "default")]
        profile: String,
    },
    Finish,
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum TicketCommands {
    Init {
        #[arg(long, short)]
        project: Option<String>,
    },
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

        #[arg(long, short = 'P')]
        project: Option<String>,
    },
    /// Run `git flow finish`
    Finish { id_or_url: Option<String> },

    /// Print userStory link
    Link { id_or_url: Option<String> },

    /// Print userStory link
    GetBranch { id_or_url: String },

    /// Print userStory ID
    GetId { url: Option<String> },

    /// Print project details
    GetProject {
        #[arg(long)]
        id: Option<String>,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        json: bool,
    },

    /// Generate a changelog from a targetprocess release
    #[clap(alias = "changelog")]
    GenerateChangelog {
        from: usize,

        to: Option<usize>,

        #[arg(long, short = 'P')]
        project: String,

        #[arg(long, short = 'p', default_value_t = String::new())]
        prefix: String,
    },
}

#[derive(Subcommand, strum::Display, Debug, Clone)]
pub enum PullRequestCommands {
    /// Create a PR
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

    #[cfg(debug_assertions)]
    CacheTest,

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
    /// Create / Manage and List pull requests
    #[clap(visible_alias = "pr")]
    PullRequest {
        #[command(subcommand)]
        subcommands: PullRequestCommands,

        /// aws profile
        #[arg(long, default_value = "default")]
        profile: String,
    },

    Bump {
        #[arg(long)]
        patch: bool,

        #[arg(long)]
        minor: bool,

        #[arg(long)]
        major: bool,
    },
}
