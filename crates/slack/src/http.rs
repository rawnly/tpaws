use color_eyre::Result;
use reqwest::header::*;

pub(crate) fn make_client() -> Result<reqwest::Client> {
    let client = reqwest::ClientBuilder::new()
        .https_only(true)
        .default_headers(get_headers())
        .build()?;

    Ok(client)
}

pub(crate) fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    headers
}
