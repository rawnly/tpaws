use crate::utils;
use color_eyre::Result;

pub async fn get_id(url: Option<String>) -> Result<()> {
    let id = utils::extract_id(url).await?;
    println!("{id}");

    Ok(())
}
