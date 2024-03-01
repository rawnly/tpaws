use color_eyre::Result;
use models::{
    assignable::{Assignable, UpdateEntityStatePayload, ID},
    user::CurrentUser,
    EntityStates,
};
use reqwest::header::*;
use serde::{de::DeserializeOwned, Serialize};

pub mod models;

pub const BASE_URL: &str = "https://satispay.tpondemand.com/api";

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

pub trait Parameter {
    fn into() -> (String, String);
}

fn make_url<I>(path: String, _params: I) -> Result<reqwest::Url>
where
    I: IntoIterator<Item = Param>,
{
    let token = get_token()?;
    let mut params: Vec<Param> = vec![Param::AccessToken(token)];
    params.extend(_params);

    let params: Vec<(String, String)> = params.into_iter().map(|p| p.into()).collect();

    let url = reqwest::Url::parse_with_params(&format!("{BASE_URL}/{path}"), params)?;

    Ok(url)
}

pub fn has_token() -> bool {
    get_token().is_ok()
}

pub async fn fetch_text<I>(path: String, params: I) -> Result<String>
where
    I: IntoIterator<Item = Param>,
{
    let client = make_client();
    let url = make_url(path, params)?;
    let headers = get_headers();

    let response = client.get(url).headers(headers).send().await?;
    let txt = response.text().await?;

    Ok(txt)
}

pub enum Param {
    Select(String),
    Where(String),
    Filter(String),
    AccessToken(String),
}

#[allow(clippy::from_over_into)]
impl Into<(String, String)> for Param {
    fn into(self) -> (String, String) {
        match self {
            Self::AccessToken(value) => ("access_token".to_string(), value),
            Self::Where(value) => ("where".to_string(), value),
            Self::Filter(value) => ("filter".to_string(), value),
            Self::Select(value) => ("select".to_string(), value),
        }
    }
}

impl From<(String, String)> for Param {
    fn from(value: (String, String)) -> Self {
        match value.0.as_str() {
            "filter" => Self::Filter(value.1),
            "where" => Self::Where(value.1),
            "select" => Self::Select(value.1),
            "access_token" => Self::AccessToken(value.1),
            _ => Self::Filter(value.1),
        }
    }
}

pub async fn fetch<T, I>(path: String, params: I) -> Result<T>
where
    T: DeserializeOwned,
    I: IntoIterator<Item = Param>,
{
    let client = make_client();
    let url = make_url(path, params)?;
    let headers = get_headers();

    let response = client.get(url).headers(headers).send().await?;
    let user = response.json::<T>().await?;

    Ok(user)
}

pub async fn get_assignable(id: &str) -> Result<Assignable> {
    let url = format!("/v1/Assignables/{id}");

    fetch(url, []).await
}

pub async fn get_me() -> Result<CurrentUser> {
    fetch("/v1/Users/loggeduser".into(), []).await
}

pub async fn get_my_tasks(current_user_id: usize) -> Result<Vec<Assignable>> {
    let filter = Param::Filter(format!("(Owner.Id = {current_user_id})"));

    fetch("/v1/Assignables".into(), vec![filter]).await
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
    let url = make_url(format!("/v1/Assignables/{assignable_id}"), [])?;

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

pub async fn update_entity_state(
    assignable_id: usize,
    entity_state_id: EntityStates,
) -> Result<Assignable> {
    let client = make_client();
    let headers = get_headers();
    let url = make_url(format!("/v1/Assignables/{assignable_id}"), [])?;

    let payload = UpdateEntityStatePayload {
        id: assignable_id,
        entity_state: ID {
            id: entity_state_id.into(),
        },
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
