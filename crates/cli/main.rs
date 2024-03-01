use clap::Parser;
use color_eyre::{eyre::OptionExt, Result};
use colored::*;
use commands::{
    aws::{PullRequest, PullRequestStatus, AWS},
    git, spawn_command,
};
use human_panic::setup_panic;
use mdka::from_html;
use spinners::Spinner;
use target_process::models::{assignable::Assignable, EntityStates};

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
        cli::Commands::PullRequest {
            subcommands,
            profile,
            ..
        } => {
            let aws = AWS::new(profile);
            aws.refresh_auth_if_needed().await?;

            let region = aws.get_region().await?;

            match subcommands {
                cli::PullRequestCommands::Create {
                    title,
                    description,
                    base,
                    no_slack,
                } => {
                    subcommands::create_pr::create_pr(
                        create_pr_args,
                        &aws,
                        title,
                        description,
                        base,
                        no_slack,
                    )
                    .await?
                }
                cli::PullRequestCommands::View { id, web } => {
                    let branch = git::current_branch().await?;
                    let repository = utils::get_repository().await?;

                    let all_prs = aws
                        .list_pull_requests(repository.clone(), PullRequestStatus::Open)
                        .await?;

                    for pr_id in all_prs.pull_request_ids {
                        let current_pr = aws.get_pull_request(pr_id).await?;
                        let link = utils::build_pr_link(
                            region.clone(),
                            repository.clone(),
                            current_pr.clone().pull_request.id.to_string(),
                        );

                        for target in current_pr.pull_request.targets {
                            if target.source.replace("refs/heads/", "") != branch {
                                continue;
                            }

                            if web {
                                println!(
                                    "Opening \"{title}\"...",
                                    title = current_pr.pull_request.title.yellow()
                                );
                                spawn_command!("open", &link)?;
                            } else {
                                println!(
                                    "[{id}] {title} - ({status})",
                                    id = current_pr.pull_request.id,
                                    title = current_pr.pull_request.title,
                                    status = current_pr.pull_request.status
                                );

                                println!();
                                println!("{}", link.blue());
                            }

                            break;
                        }
                    }
                }
                cli::PullRequestCommands::Merge { id } => {
                    let branch = git::current_branch().await?;
                    let pr = utils::get_pr_id(&aws, id).await?;
                    let repository = utils::get_repository().await?;
                    let link = utils::build_pr_link(region, repository.clone(), pr.id.to_string());

                    println!("Found 1 matching PR");
                    println!();
                    println!("[{}] {}", pr.id.yellow(), pr.title.yellow());
                    println!("{}", pr.description.yellow());
                    println!("{}", link.blue());
                    println!();

                    for target in pr.targets {
                        println!(
                            "From: {}",
                            target.source.replace("refs/heads/", "").magenta()
                        );
                        println!(
                            "To:   {}",
                            target.destination.replace("refs/heads/", "").magenta()
                        );
                        println!();
                    }

                    if !inquire::Confirm::new("Are the info above correct?")
                        .with_default(false)
                        .prompt()?
                    {
                        println!("operation aborted.");
                        return Ok(());
                    }

                    // TODO: Refactor once we have settings
                    let name = inquire::Text::new("Author Name")
                        .with_default("Federico Vitale")
                        .prompt()?;

                    // TODO: Refactor once we have settings
                    let email = inquire::Text::new("Author Email")
                        .with_default("federico.vitale@satispay.com")
                        .prompt()?;

                    let commit = inquire::Text::new("Commit Message")
                        .with_default(&pr.description)
                        .prompt()?;

                    if inquire::Confirm::new("Confirm?")
                        .with_default(false)
                        .prompt()?
                    {
                        let mut merge_spinner = Spinner::new(
                            spinners::Spinners::Dots,
                            format!("Squashing {}...", pr.id),
                        );

                        let updated_pr = aws
                            .merge_pr_by_squash(pr.id, repository, commit, name, email)
                            .await?;

                        merge_spinner.stop_with_symbol("✅");

                        if inquire::Confirm::new("Delete remote branch?")
                            .with_default(true)
                            .prompt()?
                        {
                            let mut branch_spinner =
                                Spinner::new(spinners::Spinners::Dots, "Deleting branch...".into());
                            git::delete_remote_branch("origin", branch.clone()).await?;
                            git::fetch(true).await?;
                            branch_spinner.stop_with_symbol("✅");
                        }

                        let mut spinner = Spinner::new(
                            spinners::Spinners::Dots,
                            "Updating ticket status..".to_string(),
                        );

                        if let Some(assignable_id) = utils::get_ticket_id_from_branch(branch) {
                            // TODO: remove this api call and parse assignable_id to usize
                            let ticket = target_process::get_assignable(&assignable_id).await?;

                            target_process::update_entity_state(ticket.id, EntityStates::InStaging)
                                .await?;

                            spinner.stop_with_symbol("✅");
                        } else {
                            spinner.stop_and_persist(
                                "✅",
                                "Failed to update ticket status".to_string(),
                            );
                        }

                        dbg!(&updated_pr);

                        return Ok(());
                    }

                    println!("operation aborted.");
                }
            }
        }
        cli::Commands::Ticket { subcommands } => match subcommands {
            cli::TicketCommands::Start {
                id_or_url,
                branch,
                no_git,
                no_assign,
            } => {
                let me = target_process::get_me().await?;
                let all_my_tickets = target_process::get_my_tasks(me.id).await?;
                let list: Vec<String> = all_my_tickets
                    .iter()
                    .filter_map(|t| {
                        let state = EntityStates::try_from(t.entity_state.id).ok()?;

                        if state != EntityStates::Open && state != EntityStates::Planned
                            || t.name.is_empty()
                        {
                            return None;
                        }

                        Some(t.name.clone())
                    })
                    .collect();

                if list.is_empty() {
                    dbg!(&all_my_tickets);
                    println!("No tickets are available, please provide an id");
                    return Ok(());
                }

                let id_or_url = match id_or_url {
                    Some(v) => v,
                    None => {
                        let title = inquire::Select::new("Choose a ticket", list).prompt()?;

                        let id = all_my_tickets
                            .iter()
                            .find_map(|ticket| {
                                if ticket.name == title {
                                    return Some(ticket.id);
                                }

                                None
                            })
                            .map(|id| id.to_string());

                        id.ok_or_eyre("unable to extract ticket id")?
                    }
                };

                let id = utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url);
                let assignable = target_process::get_assignable(&id).await?;
                let me = target_process::get_me().await?;

                if !no_assign {
                    let user_id = me.id;
                    let assignable_id = assignable.id;
                    target_process::assign_task(assignable_id, user_id).await?;
                    target_process::update_entity_state(assignable_id, EntityStates::InProgress)
                        .await?;
                }

                if !no_git {
                    let branch = branch.unwrap_or(assignable.get_branch());
                    commands::git::flow::feature::start(&branch).await?;
                }

                println!();
            }
            cli::TicketCommands::Finish { id_or_url } => {
                let id = if let Some(id_or_url) = id_or_url {
                    utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url)
                } else {
                    let branch = git::current_branch().await?;

                    utils::get_ticket_id_from_branch(branch)
                        .ok_or_eyre("Unable to extract userStory ID")?
                };

                let assignable = target_process::get_assignable(&id).await?;

                let branch = assignable.get_branch();
                commands::git::flow::feature::finish(&branch).await?;
            }
            cli::TicketCommands::Get { id_or_url } => {
                let id = if let Some(id_or_url) = id_or_url {
                    utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url)
                } else {
                    let branch = git::current_branch().await?;

                    utils::get_ticket_id_from_branch(branch)
                        .ok_or_eyre("Unable to extract userStory ID")?
                };

                let assignable = target_process::get_assignable(&id).await?;

                println!("{}", assignable.name);
                println!("===================");
                println!();

                match assignable.description {
                    Some(description) => print_body(description),
                    None => println!("no description provided."),
                };

                println!();
            }
            cli::TicketCommands::Open { id_or_url } => {
                let id = if let Some(id_or_url) = id_or_url {
                    utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url)
                } else {
                    let branch = git::current_branch().await?;

                    utils::get_ticket_id_from_branch(branch)
                        .ok_or_eyre("Unable to extract userStory ID")?
                };

                let assignable = target_process::get_assignable(&id).await?;

                spawn_command!("open", assignable.get_link())?;
            }
            cli::TicketCommands::Link { id_or_url } => {
                let id = if let Some(id_or_url) = id_or_url {
                    utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url)
                } else {
                    let branch = git::current_branch().await?;

                    utils::get_ticket_id_from_branch(branch)
                        .ok_or_eyre("Unable to extract userStory ID")?
                };

                let assignable = target_process::get_assignable(&id).await?;

                println!("{}", assignable.get_link());
            }
        },
    }

    Ok(())
}

fn print_body(description: String) {
    if !description.starts_with("<!--markdown-->") {
        let description = from_html(&description);

        termimad::print_text(&description);
        return;
    }

    termimad::print_text(&description.replace("<!--markdown-->", ""));
}
