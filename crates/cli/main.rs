use clap::CommandFactory;
use clap::Parser;
use color_eyre::{
    eyre::{eyre, OptionExt},
    Result,
};
use colored::*;
use commands::{
    aws,
    git::{self},
    spawn_command,
};
use config::{util::get_user_id, Config, ProjectConfig};
use global_utils::print_dbg;
use human_panic::setup_panic;
use mdka::from_html;
use target_process::{models::EntityStates, SearchOperator};

use crate::{
    cli::Args, context::GlobalContext, manifests::node::PackageJson,
    subcommands::release::ReleaseKind, telemetry::track_event,
};

mod cli;
mod context;
mod costants;
mod manifests;
mod subcommands;
mod telemetry;
mod utils;

fn print_help() -> Result<()> {
    let mut cmd = Args::command();
    cmd.print_help()?;

    Ok(())
}

fn is_slack_enabled(slack_flag: bool) -> bool {
    if cfg!(debug_assertions) {
        return slack_flag;
    }

    false
}

#[tokio::main]
#[allow(unreachable_code, unused_variables)]
async fn main() -> Result<()> {
    telemetry::init()?;
    let axiom = axiom_rs::Client::new()?;

    //  TODO: replace with axiom
    let _guard = sentry::init((
        env!("SENTRY_DSN"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    ));

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            id: get_user_id(),
            ..Default::default()
        }))
    });
    // end TODO

    setup_panic!();

    #[cfg(debug_assertions)]
    color_eyre::install()?;

    let args = cli::Args::parse();

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.debug {
        std::env::set_var("TPAWS_DEBUG", args.debug.to_string());
    }

    if args.command.is_none() {
        return print_help();
    }

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

    if Config::is_first_run()? {
        println!("Please configure the CLI before continue");
        println!();

        telemetry::track_event(telemetry::Event::Installation, Some(""))?;

        subcommands::config::reset().await?;
    }

    let config = Config::read().await?;
    let local_config = ProjectConfig::read().await;

    if !target_process::has_token() {
        println!("No env for {}", target_process::ENV_NAME);
        return Ok(());
    }

    let create_pr_args = args.clone();

    match args.command.unwrap() {
        cli::Commands::PullRequest {
            subcommands,
            profile,
            ..
        } => {
            let branch = git::current_branch_v2().await?.0;
            let repository = utils::get_repository().await?;

            let region = aws::get_region(profile.clone()).await?;

            let mut ctx = GlobalContext::new(
                profile,
                region.clone(),
                config,
                branch.clone(),
                repository.clone(),
            );

            if ctx.config.is_auth_expired() {
                let arn = aws::refresh_auth_if_needed(ctx.profile.clone()).await?;
                ctx.config.update_auth(arn);
                ctx.config.write()?;
            }

            match subcommands {
                cli::PullRequestCommands::List {
                    interactive,
                    status,
                } => subcommands::pull_request::list(ctx, status, interactive).await?,
                cli::PullRequestCommands::Create {
                    title,
                    description,
                    base,
                    slack,
                } => {
                    subcommands::pull_request::create(
                        ctx,
                        create_pr_args,
                        title,
                        description,
                        base,
                        is_slack_enabled(slack),
                    )
                    .await?
                }
                cli::PullRequestCommands::View { id, web } => {
                    subcommands::pull_request::view(ctx, id, web).await?
                }
                cli::PullRequestCommands::Merge {
                    id,
                    author,
                    email: author_email,
                    commit_message,
                    ..
                } => {
                    subcommands::pull_request::merge(
                        ctx,
                        id,
                        author_email,
                        commit_message,
                        region,
                        utils::RepoMetadata::new(repository, branch),
                        args.quiet,
                    )
                    .await?
                }
            }
        }
        #[cfg(debug_assertions)]
        cli::Commands::CacheTest => {
            for i in 0..10 {
                let start = std::time::Instant::now();
                let _ = target_process::get_me().await?;

                println!("{:?}", start.elapsed());
            }
        }
        cli::Commands::Ticket { subcommands } => match subcommands {
            cli::TicketCommands::Start {
                id_or_url,
                branch,
                no_git,
                no_assign,
                project,
            } => {
                let project = match local_config.and_then(|c| c.tp_name).or(project) {
                    Some(p) => p,
                    None => return Err(eyre!("Unable to extract project")),
                };

                let all_my_tickets =
                    target_process::get_current_sprint_open_tasks(&project).await?;

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

                let id_or_url = match id_or_url {
                    Some(v) => v,
                    None => {
                        if list.is_empty() {
                            global_utils::print_dbg!(&all_my_tickets);
                            println!("No tickets are available, please provide an id");
                            return Ok(());
                        }

                        let title = inquire::Select::new("Pick a user story from the list:", list)
                            .prompt()?;

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
                let assignable = target_process::get_assignable(id).await?;

                if !no_assign {
                    let user_id = config.user_id;
                    let assignable_id = assignable.id;
                    target_process::assign_task(assignable_id, user_id).await?;

                    if assignable.is_user_story() {
                        target_process::update_entity_state(
                            assignable_id,
                            EntityStates::InProgress,
                        )
                        .await?;
                    }
                }

                if !no_git {
                    let branch = branch.unwrap_or(assignable.get_branch());
                    commands::git::flow::feature::start(&branch).await?;
                }
            }
            cli::TicketCommands::Finish { id_or_url } => {
                let id = if let Some(id_or_url) = id_or_url {
                    utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url)
                } else {
                    let branch = git::current_branch_v2().await?.0;

                    utils::get_ticket_id_from_branch(branch)
                        .ok_or_eyre("Unable to extract userStory ID")?
                };

                let assignable = target_process::get_assignable(id).await?;

                let branch = assignable.get_branch();
                commands::git::flow::feature::finish(&branch).await?;
            }
            cli::TicketCommands::View {
                id_or_url,
                json,
                web,
            } => {
                let id = if let Some(id_or_url) = id_or_url {
                    utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url)
                } else {
                    let branch = git::current_branch_v2().await?.0;

                    utils::get_ticket_id_from_branch(branch)
                        .ok_or_eyre("Unable to extract userStory ID")?
                };

                let assignable = target_process::get_assignable(id).await?;

                if web {
                    spawn_command!("open", assignable.get_link())?;
                    return Ok(());
                }

                if json {
                    let json_string = serde_json::to_string_pretty(&assignable)?;
                    println!("{}", json_string);

                    return Ok(());
                }

                println!("{}", assignable.name);
                println!("===================");
                println!();

                match assignable.description {
                    Some(description) => print_ticket_body(description),
                    None => println!("no description provided."),
                };

                println!();
            }
            cli::TicketCommands::Link { id_or_url } => {
                let id = if let Some(id_or_url) = id_or_url {
                    utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url)
                } else {
                    let branch = git::current_branch_v2().await?.0;

                    utils::get_ticket_id_from_branch(branch)
                        .ok_or_eyre("Unable to extract userStory ID")?
                };

                let assignable = target_process::get_assignable(id).await?;

                println!("{}", assignable.get_link());
            }
            cli::TicketCommands::GetBranch { id_or_url } => {
                let id = utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url);
                let assignable = target_process::get_assignable(id).await?;

                println!("{}", assignable.get_branch());
            }
            cli::TicketCommands::GetId { url } => {
                let id = if let Some(url) = url {
                    utils::extract_id_from_url(url.clone()).unwrap_or(url)
                } else {
                    let branch = git::current_branch_v2().await?.0;

                    utils::get_ticket_id_from_branch(branch)
                        .ok_or_eyre("Unable to extract userStory ID")?
                };

                println!("{id}");
            }
            cli::TicketCommands::GetProject { id, json, name } => {
                let branch = git::current_branch_v2().await?;

                match id {
                    Some(id) => {
                        let project = target_process::get_project(id).await?;

                        if json {
                            println!("{}", serde_json::to_string_pretty(&project)?);
                            return Ok(());
                        }

                        println!("Id: {}", project.id);
                        println!("Name: {}", project.name);
                        println!(
                            "Url: {}/entities/{}",
                            target_process::get_base_url(),
                            project.id
                        );
                    }
                    None => match name {
                        Some(name) => {
                            let projects =
                                target_process::search_project(name, SearchOperator::Eq).await?;
                            let project = projects.first().ok_or_eyre("failed to find project")?;

                            if json {
                                let json_pretty = serde_json::to_string_pretty(&project)?;
                                println!("{json_pretty}");
                                return Ok(());
                            }

                            println!("Id: {}", project.id);
                            println!("Name: {}", project.name);
                            println!(
                                "Url: {}/entities/{}",
                                target_process::get_base_url(),
                                project.id
                            );

                            return Ok(());
                        }
                        None => {
                            if !branch.is_feature() {
                                println!("Please make sure to be in a feature branch or pass a valid project id or name");
                                return Ok(());
                            }

                            let assignable_id = utils::get_ticket_id_from_branch(branch.0)
                                .ok_or_eyre("unable to extract user story ID")?;
                            let assignable = target_process::get_assignable(assignable_id).await?;
                            let project = assignable.project;

                            if json {
                                let json_pretty = serde_json::to_string_pretty(&project)?;
                                println!("{json_pretty}");
                                return Ok(());
                            }

                            println!("Id: {}", project.id);
                            println!("Name: {}", project.name);
                            println!(
                                "Url: {}/entities/{}",
                                target_process::get_base_url(),
                                project.id
                            );
                        }
                    },
                }
            }
            cli::TicketCommands::GenerateChangelog {
                from,
                to,
                prefix,
                project,
            } => {
                let changelog =
                    target_process::generate_changelog(from, to, project, prefix).await?;

                if changelog.is_empty() {
                    println!("Empty changelog :(");
                    return Ok(());
                }

                for str in changelog {
                    println!("{str}")
                }
            }
        },
        cli::Commands::Config { subcommands } => match subcommands {
            cli::ConfigCommands::Reset => subcommands::config::reset().await?,
        },
        cli::Commands::Release { subcommands } => {
            if !PackageJson::exists().await {
                return Err(eyre!("currently we support only nodejs"));
            }

            let pkg = PackageJson::read().await?;
            let branch = git::current_branch_v2().await?;

            match subcommands {
                cli::ReleaseCommands::Start {
                    major,
                    patch,
                    minor,
                } => {
                    let kind: ReleaseKind = match (major, minor, patch) {
                        (_, _, true) => ReleaseKind::Patch,
                        (true, _, _) => ReleaseKind::Major,
                        (_, true, _) => ReleaseKind::Minor,
                        _ => ReleaseKind::Minor,
                    };

                    print_dbg!(&kind, major, minor, patch);

                    subcommands::release::start(&pkg, kind).await?;
                }
                cli::ReleaseCommands::Push {
                    target,
                    pipeline_name: name,
                    profile,
                } => {
                    subcommands::release::push(target, name, profile).await?;
                }
                cli::ReleaseCommands::Finish => {
                    subcommands::release::finish(&pkg, &branch.0).await?;
                }
            }
        }
    }

    Ok(())
}

fn print_ticket_body(description: String) {
    if !description.starts_with("<!--markdown-->") {
        let description = from_html(&description);

        termimad::print_text(&description);
        return;
    }

    termimad::print_text(&description.replace("<!--markdown-->", ""));
}
