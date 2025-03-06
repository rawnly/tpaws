use ai::groq::models;
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
};
use config::{util::get_user_id, Config, ProjectConfig};
use human_panic::setup_panic;
use target_process::models::EntityStates;

use crate::{cli::Args, context::GlobalContext, subcommands::user_story};

mod cli;
mod context;
mod costants;
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
    let axiom_token = env!("AXIOM_TOKEN");

    let axiom = axiom_rs::Client::builder()
        .with_token(axiom_token)
        .build()
        .expect("failed to initialize axiom");

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
        telemetry::track_event(telemetry::Event::NoAwsInstalled, Some("")).await?;

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

        telemetry::track_event(telemetry::Event::Installation, Some("")).await?;

        subcommands::config::reset().await?;
    }

    let mut config = Config::read().await?;
    let local_config = ProjectConfig::read().await;

    if !target_process::has_token() {
        println!("No env for {}", target_process::ENV_NAME);
        return Ok(());
    }

    let create_pr_args = args.clone();
    let groq_api_key = config
        .clone()
        .groq_api_key
        .or_else(models::get_apikey_from_env);

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
                    copy,
                    ai,
                    ai_model,
                } => {
                    subcommands::pull_request::create(
                        &mut ctx,
                        create_pr_args,
                        title,
                        description,
                        base,
                        is_slack_enabled(slack),
                        copy,
                        ai,
                        ai_model,
                    )
                    .await?
                }
                cli::PullRequestCommands::View {
                    id,
                    web,
                    copy_url,
                    markdown,
                } => subcommands::pull_request::view(ctx, id, web, copy_url, markdown).await?,
            }
        }
        cli::Commands::Ticket { subcommands } => match subcommands {
            cli::TicketCommands::Init { project } => {
                let projects = target_process::get_projects(0, 200).await?;

                let list: Vec<String> = projects
                    .iter()
                    .map(|p| {
                        p.abbreviation
                            .as_ref()
                            .map_or(p.name.clone(), |abbr| format!("{} - {}", abbr, p.name))
                    })
                    .collect();

                let picked =
                    inquire::Select::new("Pick a project from the list:", list).prompt()?;

                let project = match picked.contains('-') {
                    true => {
                        let abbr = picked.split(" - ").next().unwrap();
                        let name = picked.split(" - ").last().unwrap();

                        projects.iter().find(|p| {
                            p.abbreviation.as_ref().is_none_or(|a| a == abbr) && p.name == name
                        })
                    }
                    false => projects.iter().find(|p| p.name == picked),
                }
                .ok_or(eyre!("unable to find project"))?;

                println!("Project: {}", project.name);
            }
            cli::TicketCommands::Start {
                id_or_url,
                branch,
                no_git,
                no_assign,
                project,
            } => {
                let project = match local_config.and_then(|c| c.name).or(project) {
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
            cli::TicketCommands::View {
                id_or_url,
                json,
                web,
            } => user_story::view(id_or_url, json, web).await?,
            cli::TicketCommands::Link { id_or_url } => user_story::link(id_or_url).await?,
            cli::TicketCommands::GetBranch { id_or_url } => {
                user_story::get_branch(id_or_url).await?
            }
            cli::TicketCommands::GetId { url } => user_story::get_id(url).await?,
            cli::TicketCommands::GenerateCommit {
                id_or_url,
                json,
                title_only,
            } => user_story::generate_commit(id_or_url, json, title_only, &mut config).await?,
            cli::TicketCommands::GenerateChangelog {
                from,
                to,
                prefix,
                project,
                plain,
                no_title,
            } => {
                let changelog =
                    target_process::generate_changelog(from, to, project, prefix, plain, no_title)
                        .await?;

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
        cli::Commands::Init { project, force } => {
            if ProjectConfig::exists() && !force {
                println!("Project already initialized");
                return Ok(());
            }

            let config = ProjectConfig {
                name: project.or_else(|| inquire::Text::new("Project name:").prompt().ok()),
            };

            if !args.dry_run {
                config.write()?;
            } else {
                println!("{:?}", config);
            }

            println!("Project initialized");
        }
    }

    Ok(())
}
