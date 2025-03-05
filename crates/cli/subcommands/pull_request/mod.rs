mod create;
mod list;
mod merge;
mod view;

use color_eyre::Result;
pub use create::create;
pub use list::list;
pub use merge::merge;
pub use view::view;

use commands::aws::{self, PullRequest, PullRequestStatus};

pub async fn get_current_pr(
    branch: String,
    repository: String,
    profile: String,
) -> Result<Option<PullRequest>> {
    let mut pr: Option<PullRequest> = None;

    let all_prs =
        aws::list_pull_requests(repository.clone(), PullRequestStatus::Open, profile.clone())
            .await?;

    for pr_id in all_prs.pull_request_ids {
        let current_pr = aws::get_pull_request(pr_id, profile.clone()).await?;

        for target in current_pr.clone().pull_request.targets {
            if target.source.replace("refs/heads/", "") != branch {
                continue;
            }

            pr = Some(current_pr.pull_request);

            break;
        }
    }

    Ok(pr)
}
