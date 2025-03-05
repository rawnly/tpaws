use arboard::Clipboard;
use color_eyre::Result;
use colored::*;
use commands::{
    aws::{self, PullRequest, PullRequestStatus},
    spawn_command,
};

use crate::{context::GlobalContext, utils};

use super::get_current_pr;

pub async fn view(
    ctx: GlobalContext,
    id: Option<String>,
    web: bool,
    copy_url: bool,
    markdown: bool,
) -> Result<()> {
    let GlobalContext {
        branch,
        repository,
        profile,
        region,
        ..
    } = ctx;

    let pull_request: Option<PullRequest>;

    if let Some(id) = id {
        pull_request = Some(
            aws::get_pull_request(id, profile.clone())
                .await?
                .pull_request,
        );
    } else {
        pull_request = get_current_pr(branch, repository.clone(), profile).await?;
    }

    if let Some(pull_request) = pull_request {
        let link = utils::build_pr_link(region, repository, pull_request.id.to_string());

        if copy_url {
            let mut clipboard = Clipboard::new()?;

            if markdown {
                clipboard.set_text(format!(
                    "[{}: {}]({link})",
                    pull_request.id, pull_request.title
                ))?;
            } else {
                clipboard.set_text(link.clone())?;
            }

            println!("Link to clipboard: {}", link.blue());
        } else if web {
            println!(
                "Opening \"{title}\"...",
                title = pull_request.title.yellow()
            );
            spawn_command!("open", &link)?;
        } else {
            println!(
                "[{id}] {title} - ({status})",
                id = pull_request.id,
                title = pull_request.title,
                status = pull_request.status
            );

            println!();
            println!("{}", link.blue());
        }
    }

    Ok(())
}
