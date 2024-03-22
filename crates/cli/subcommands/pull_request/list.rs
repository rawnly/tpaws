use colored::*;
use std::sync::Arc;

use color_eyre::Result;
use commands::aws::{PullRequestResponse, PullRequestStatus, PullRequestsList};
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
    } = ctx
        .aws
        .list_my_pull_requests(
            ctx.repository.clone(),
            status,
            ctx.config.clone().arn.unwrap(),
        )
        .await?;

    let mut handles = JoinSet::new();

    for id in data {
        let aws = Arc::new(ctx.aws.clone());
        let repository = Arc::new(ctx.clone().repository);

        handles.spawn(async move {
            let repository = Arc::clone(&repository);
            let aws = Arc::clone(&aws);

            if let Ok(PullRequestResponse { pull_request: pr }) =
                aws.get_pull_request(id.clone()).await
            {
                let region = aws.region.clone();
                let link = build_pr_link(region.unwrap(), repository.to_string(), id);

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
