use colored::*;
use std::sync::Arc;

use color_eyre::Result;
use commands::aws::{self, PullRequestResponse, PullRequestStatus, PullRequestsList};
use tokio::task::JoinSet;

use crate::{context::GlobalContext, utils::build_pr_link};

pub async fn list(
    ctx: GlobalContext,
    status: Option<PullRequestStatus>,
    interactive: bool,
) -> Result<()> {
    if interactive {
        unimplemented!("`--interactive` not implemented");
    }

    let PullRequestsList {
        pull_request_ids: data,
    } = aws::list_my_pull_requests(
        ctx.repository.clone(),
        status,
        ctx.config.clone().arn.unwrap(),
        ctx.profile.clone(),
    )
    .await?;

    let mut handles = JoinSet::new();

    let repository = Arc::new(ctx.repository);
    let profile = Arc::new(ctx.profile);
    let region = Arc::new(ctx.region);

    for id in data {
        let repository = Arc::clone(&repository);
        let profile = Arc::clone(&profile);
        let region = Arc::clone(&region);

        handles.spawn(async move {
            if let Ok(PullRequestResponse { pull_request: pr }) =
                aws::get_pull_request(id.clone(), profile.to_string()).await
            {
                let link = build_pr_link(region.to_string(), repository.to_string(), id);

                println!(
                    "[{status}] {id} - {title}\n\t- {link}",
                    id = pr.id.green(),
                    title = pr.title.trim(),
                    status = pr.status.bold().yellow(),
                    link = link.blue()
                );
            }
        });
    }

    while handles.join_next().await.is_some() {}

    Ok(())
}
