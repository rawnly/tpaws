use crate::utils;
use color_eyre::Result;

pub async fn link(id_or_url: Option<String>) -> Result<()> {
    let id = utils::extract_id(id_or_url).await?;
    let assignable = target_process::get_assignable(id).await?;

    println!("{}", assignable.get_link());

    Ok(())
}
