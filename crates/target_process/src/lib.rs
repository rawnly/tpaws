use cached::proc_macro::cached;

use color_eyre::eyre::Context;
use errors::*;
use models::{
    user::CurrentUser,
    v1::assignable::{Assignable, UpdateEntityStatePayload, ID},
    v2, EntityStates,
};
use reqwest::header::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod errors;
pub mod models;

type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ResponseListV1<T> {
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseListV2<T> {
    pub items: Vec<T>,
}

fn make_client() -> reqwest::Client {
    reqwest::Client::new()
}

fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    headers
}

fn get_token() -> Result<String> {
    let token = std::env::var("TARGET_PROCESS_ACCESS_TOKEN")
        .map_err(|source| ApiError::TokenNotFound { source })?;

    Ok(token)
}

pub trait Parameter {
    fn into() -> (String, String);
}

pub const ENV_NAME: &'static str = "TARGET_PROCESS_API_BASE_URL";

pub fn is_configured() -> bool {
    let value = std::env::var(ENV_NAME).ok();

    value.is_some()
}

fn make_url<I>(path: String, _params: I) -> Result<reqwest::Url>
where
    I: IntoIterator<Item = Param>,
{
    let base = std::env::var(ENV_NAME)
        .context("Unable to retrive `TARGET_PROCESS_API_BASE_URL` env variable.")
        .unwrap();

    let token = get_token()?;
    let mut params: Vec<Param> = vec![Param::AccessToken(token)];
    params.extend(_params);

    let params: Vec<(String, String)> = params.into_iter().map(|p| p.into()).collect();

    let base_url = format!("{base}/api");

    let url =
        reqwest::Url::parse_with_params(&format!("{base_url}/{path}").replace("//", "/"), params)
            .map_err(|_| ApiError::UrlParsing)?;

    Ok(url)
}

pub fn has_token() -> bool {
    get_token().is_ok()
}

#[derive(Debug, Clone)]
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

pub async fn post<T: DeserializeOwned, P: Serialize>(path: String, payload: P) -> Result<T> {
    let client = make_client();
    let headers = get_headers();
    let url = make_url(path, [])?;

    let response = client
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|e| ApiError::GenericError(e.to_string()))?;

    response
        .json::<T>()
        .await
        .map_err(|e| ApiError::Json(e.to_string()))
}

pub async fn fetch<T, I>(path: String, params: I) -> Result<T>
where
    I: IntoIterator<Item = Param> + Clone,
    T: DeserializeOwned + Clone,
{
    let client = make_client();
    let url = make_url(path, params)?;
    let headers = get_headers();

    if cfg!(debug_assertions) {
        println!("GET {url}");
    }

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| ApiError::GenericError(e.to_string()))?;

    response
        .json::<T>()
        .await
        .map_err(|e| ApiError::Json(e.to_string()))
}

#[cached]
pub async fn get_assignable(id: String) -> Result<Assignable> {
    let url = format!("/v1/Assignables/{id}");

    fetch(url, []).await
}

#[cached]
pub async fn get_me() -> Result<CurrentUser> {
    fetch("/v1/Users/loggeduser".into(), []).await
}

pub async fn get_current_sprint_open_tasks(project_name: &str) -> Result<Vec<Assignable>> {
    let where_filter = Param::Where(format!(
        r#"(EntityState.IsInitial = true) and ( EntityType.Name = 'Bug' or (TeamIteration.IsCurrent=true or TeamIteration.IsPrevious = true))and(Project.Name='{project_name}')"#
    ));

    let select_filter =
        Param::Select("{id,name,description,resourceType,entityState,entityType}".into());

    let api_response: ResponseListV2<v2::assignable::Assignable> =
        fetch("/v2/assignables".into(), vec![where_filter, select_filter]).await?;

    Ok(api_response
        .items
        .into_iter()
        .map(Assignable::from)
        .collect())
}

pub async fn assign_task(assignable_id: usize, user_id: usize) -> Result<Assignable> {
    let payload = AssignDeveloperPayload {
        assignments: vec![AssignedUser {
            role: ID { id: 1 },
            general_user: ID { id: user_id },
        }],
    };

    post(format!("/v1/Assignables/{assignable_id}"), payload).await
}

pub async fn update_entity_state(
    assignable_id: usize,
    entity_state_id: EntityStates,
) -> Result<Assignable> {
    let payload = UpdateEntityStatePayload {
        id: assignable_id,
        entity_state: ID {
            id: entity_state_id.into(),
        },
    };

    post(format!("/v1/Assignables/{assignable_id}"), payload).await
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

pub fn get_base_url() -> String {
    std::env::var(ENV_NAME).unwrap()
}
