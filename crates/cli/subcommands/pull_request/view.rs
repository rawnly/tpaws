use color_eyre::Result;
use colored::*;
use commands::{
    aws::{PullRequest, PullRequestStatus},
    spawn_command,
};

use crate::{context::GlobalContext, utils};

pub async fn view(ctx: GlobalContext, id: Option<String>, web: bool) -> Result<()> {
    let GlobalContext {
        aws,
        branch,
        repository,
    } = ctx;

    let mut pull_request: Option<PullRequest> = None;

    if let Some(id) = id {
        pull_request = Some(aws.get_pull_request(id).await?.pull_request);
    } else {
        let all_prs = aws
            .list_pull_requests(repository.clone(), PullRequestStatus::Open)
            .await?;

        for pr_id in all_prs.pull_request_ids {
            let current_pr = aws.get_pull_request(pr_id).await?;

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
        let link =
            utils::build_pr_link(aws.region.unwrap(), repository, pull_request.id.to_string());

        if web {
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
