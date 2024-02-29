use clap::Parser;
use color_eyre::Result;
use colored::*;
use human_panic::setup_panic;
use mdka::from_html;
use target_process::models::assignable::Assignable;

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

    if !commands::is_installed!("aws") {
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
        cli::Commands::Ticket { subcommands } => match subcommands {
            cli::TicketCommands::Start {
                id_or_url,
                branch,
                no_git,
                no_assign,
            } => {
                let id = utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url);
                let assignable = target_process::get_assignable(&id).await?;
                let me = target_process::get_me().await?;

                if !no_assign {
                    let user_id = me.id;
                    let assignable_id = assignable.id;
                    target_process::assign_task(assignable_id, user_id).await?;
                }

                if !no_git {
                    let branch = branch.unwrap_or(utils::branch_name(assignable));
                    commands::git::flow::feature::start(&branch).await?;
                }

                println!();
            }
            cli::TicketCommands::Get { id_or_url, json } => {
                let id = utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url);
                let assignable = target_process::get_assignable(&id).await?;

                println!();
                print_body(&assignable);
                println!();
            }
        },
    }

    Ok(())
}

fn print_body(assignable: &Assignable) {
    if !assignable.description.starts_with("<!--markdown-->") {
        let description = from_html(&assignable.description);

        termimad::print_text(&description);
        return;
    }

    termimad::print_text(&assignable.description.replace("<!--markdown-->", ""));
}
