use clap::Parser;
use color_eyre::Result;
use colored::*;
use human_panic::setup_panic;

mod cli;
mod costants;
mod subcommands;
mod utils;

#[tokio::main]
#[allow(unreachable_code, unused_variables)]
async fn main() -> Result<()> {
    setup_panic!();

    #[cfg(debug_assertions)]
    color_eyre::install()?;

    if !commands::has("aws") {
        println!("Useful links:");
        println!(
            "- {}",
            "https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html"
                .yellow()
        );
        println!();

        return Ok(());
    }

    let args = cli::Args::parse();

    let create_pr_args = args.clone();

    match args.command {
        cli::Commands::CreatePR {
            title,
            description,
            base,
            profile,
            no_slack,
        } => {
            subcommands::create_pr::create_pr(
                create_pr_args,
                title,
                description,
                base,
                profile,
                no_slack,
            )
            .await?;
        }
        _ => unimplemented!("wait!"),
    }

    Ok(())
}
