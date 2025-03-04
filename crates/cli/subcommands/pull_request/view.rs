use arboard::Clipboard;
use color_eyre::Result;
use colored::*;
use commands::{
    aws::{self, PullRequest, PullRequestStatus},
    spawn_command,
};

use crate::{context::GlobalContext, utils};

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

    let mut pull_request: Option<PullRequest> = None;

    if let Some(id) = id {
        pull_request = Some(
            aws::get_pull_request(id, profile.clone())
                .await?
                .pull_request,
        );
    } else {
        let all_prs =
            aws::list_pull_requests(repository.clone(), PullRequestStatus::Open, profile.clone())
                .await?;

        for pr_id in all_prs.pull_request_ids {
            let current_pr = aws::get_pull_request(pr_id, profile.clone()).await?;

            for target in current_pr.clone().pull_request.targets {
                if target.source.replace("refs/heads/", "") != branch {
                    continue;
                }

                pull_request = Some(current_pr.pull_request);

                break;
            }
        }
    }

    if let Some(pull_request) = pull_request {
        let link = utils::build_pr_link(region, repository, pull_request.id.to_string());

        if copy_url {
            let mut clipboard = Clipboard::new()?;

            if markdown {
                clipboard.set_text(format!(
                    "[{}: {}]({link})",
                    pull_request.id, pull_request.title
                ));
            } else {
                clipboard.set_text(link.clone());
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
