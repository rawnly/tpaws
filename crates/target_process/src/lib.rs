use color_eyre::Result;
use models::{assignable::Assignable, user::CurrentUser};
use reqwest::header::*;
use serde::{de::DeserializeOwned, Serialize};

pub mod models;

pub const BASE_URL: &str = "https://satispay.tpondemand.com/api/v1";

fn make_client() -> reqwest::Client {
    reqwest::Client::new()
}

fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    headers
}

fn get_token() -> Result<String> {
    let token = std::env::var("TARGET_PROCESS_ACCESS_TOKEN")?;

    Ok(token)
}

fn make_url(path: String, filter: Option<String>) -> Result<reqwest::Url> {
    let token = get_token()?;
    let mut params: Vec<(String, String)> = vec![("access_token".to_string(), token)];

    if let Some(filter) = filter {
        params.push(("where".to_string(), filter))
    }

    let url = reqwest::Url::parse_with_params(&format!("{BASE_URL}/{path}"), &params)?;

    Ok(url)
}

pub fn has_token() -> bool {
    get_token().is_ok()
}

pub async fn fetch_text(path: String, filter: Option<String>) -> Result<String> {
    let client = make_client();
    let url = make_url(path, filter)?;
    let headers = get_headers();

    let response = client.get(url).headers(headers).send().await?;
    let txt = response.text().await?;

    Ok(txt)
}

pub async fn fetch<T: DeserializeOwned>(path: String, filter: Option<String>) -> Result<T> {
    let client = make_client();
    let url = make_url(path, filter)?;
    let headers = get_headers();

    let response = client.get(url).headers(headers).send().await?;
    let user = response.json::<T>().await?;

    Ok(user)
}

pub async fn get_assignable(id: &str) -> Result<Assignable> {
    let url = format!("Assignables/{id}");

    fetch(url, None).await
}

pub async fn get_me() -> Result<CurrentUser> {
    fetch("Users/loggeduser".into(), None).await
}

pub async fn get_my_tasks(current_user_id: usize) -> Result<Vec<Assignable>> {
    let filter = format!("(Owner.Id = {current_user_id})");

    fetch("Assignables".into(), Some(filter)).await
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ID {
    pub id: usize,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct AssignDeveloperPayload {
    pub assignments: Vec<AssignedUser>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct AssignedUser {
    pub general_user: ID,
    pub role: ID,
}

pub async fn assign_task(assignable_id: usize, user_id: usize) -> Result<Assignable> {
    let client = make_client();
    let headers = get_headers();
    let url = make_url(format!("/Assignables/{assignable_id}"), None)?;

    let payload = AssignDeveloperPayload {
        assignments: vec![AssignedUser {
            role: ID { id: 1 },
            general_user: ID { id: user_id },
        }],
    };

    let response = client
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    if cfg!(debug_assertions) {
        dbg!(&response);
    }

    let assignable: Assignable = response.json().await?;

    Ok(assignable)
}
