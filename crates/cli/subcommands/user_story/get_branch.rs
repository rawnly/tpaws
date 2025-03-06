use crate::utils;
use color_eyre::Result;

pub async fn get_branch(id_or_url: String) -> Result<()> {
    let id = utils::extract_id_from_url(id_or_url.clone()).unwrap_or(id_or_url);
    let assignable = target_process::get_assignable(id).await?;

    println!("{}", assignable.get_branch());

    Ok(())
}
