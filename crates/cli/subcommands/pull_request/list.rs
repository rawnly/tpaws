use std::sync::Arc;

use color_eyre::Result;
use commands::aws::PullRequestsList;
use tokio::task::JoinSet;

use crate::context::GlobalContext;

pub async fn list(ctx: GlobalContext, interactive: bool) -> Result<()> {
    if interactive {
        unimplemented!("`--interactive` not implemented");
    }

    let PullRequestsList {
        pull_request_ids: data,
    } = ctx
        .aws
        .list_my_pull_requests(ctx.repository.clone(), None, ctx.aws.profile.clone())
        .await?;

    let mut handles = JoinSet::new();

    for id in data {
        let aws = Arc::new(ctx.aws.clone());
        handles.spawn(async move { Arc::clone(&aws).get_pull_request(id).await });
    }

    while let Some(Ok(Ok(pr))) = handles.join_next().await {
        println!(
            "{id} - {title}",
            id = pr.pull_request.id,
            title = pr.pull_request.title
        );
    }

    Ok(())
}
