use std::sync::{Arc, Mutex};

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

use crate::models::v1::assignable::Project;

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

pub const ENV_NAME: &str = "TARGET_PROCESS_API_BASE_URL";

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

    let text = response
        .text()
        .await
        .map_err(|e| ApiError::GenericError(e.to_string()))?;

    serde_json::from_str(&text).map_err(|e| ApiError::Json(e.to_string()))
}

#[derive(Debug)]
enum Row {
    Title(String),
    Log(String, String),
}

impl ToString for Row {
    fn to_string(&self) -> String {
        let base_url = get_base_url();

        match self {
            Self::Title(version) => format!("## {version}"),
            Self::Log(id, name) => format!("- [{id}]({base_url}/entity/{id}) {name}"),
        }
    }
}

#[cached]
pub async fn generate_changelog(
    from: usize,
    to: Option<usize>,
    project_name: String,
    release_prefix: String,
) -> Result<Vec<String>> {
    let to = to.unwrap_or(from);

    let prefix = Arc::new(release_prefix);
    let project = Arc::new(project_name);
    let range = (from..to + 1).rev();
    let rows: Arc<Mutex<Vec<Row>>> = Arc::new(Mutex::new(Vec::new()));

    for minor in range {
        let rows = Arc::clone(&rows);
        let project = Arc::clone(&project);
        let prefix = Arc::clone(&prefix);
        let prefix = Arc::clone(&prefix);

        tokio::spawn(async move {
            let version = Arc::new(format!("1.{minor}"));

            for patch in 0..10 {
                // let rows = Arc::clone(&rows);
                let project = Arc::clone(&project);
                let prefix = Arc::clone(&prefix);
                let version = Arc::clone(&version);

                let version = format!("{version}.{patch}");

                let elements = get_tag_tickets(project, prefix, version.clone())
                    .await
                    .unwrap();

                if elements.is_empty() {
                    return;
                }

                if let Ok(mut v) = rows.lock() {
                    let row = Row::Title(version);
                    v.push(row);

                    for (id, name) in elements {
                        let row = Row::Log(id, name);
                        v.push(row);
                    }
                }
            }
        })
        .await
        .map_err(|e| ApiError::GenericError(e.to_string()))?;
    }

    let rows = rows.lock().unwrap();
    let strings: Vec<String> = rows.iter().map(|row| row.to_string()).collect();

    Ok(strings)
}

#[cached]
async fn get_tag_tickets(
    project: Arc<String>,
    release_prefix: Arc<String>,
    version: String,
) -> Result<Vec<(String, String)>> {
    let mut data = vec![];
    let release_name = if release_prefix.is_empty() {
        version.to_string()
    } else {
        format!("{release_prefix}@{version}")
    };

    let filter = format!("(Project.Name='{project}')and(Release.Name='{release_name}')");

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v2/assignables", get_base_url()))
        .header("Accept", "application/json")
        .query(&[
            ("access_token", get_token()?),
            ("where", filter),
            ("select", "{id,name}".to_string()),
        ])
        .send()
        .await
        .map_err(|e| ApiError::GenericError(e.to_string()))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::Json(e.to_string()))?;

    if let Some(response) = json.as_object() {
        if let Some(v) = response.get("items") {
            if let Some(items) = v.as_array() {
                for item in items {
                    if let Some(item_obj) = item.as_object() {
                        let id = item_obj.get("id").unwrap();
                        let name = item_obj.get("name").unwrap();

                        data.push((id.to_string(), name.to_string()));
                    }
                }
            }
        }
    }

    Ok(data)
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignablesList {
    pub items: serde_json::Value,
}

#[cached]
pub async fn get_project(id: String) -> Result<Project> {
    let url = format!("/v1/Projects/{id}");

    fetch(url, []).await
}

#[cached]
pub async fn get_assignables(filter: String, select: String) -> Result<AssignablesList> {
    let url = String::from("/v2/assignables");

    fetch(url, [Param::Filter(filter), Param::Select(select)]).await
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

#[derive(strum::Display, PartialEq, Eq, Clone, Hash)]
pub enum SearchOperator {
    Eq,
    Contains,
}

#[cached]
pub async fn search_project(name: String, operator: SearchOperator) -> Result<Vec<Project>> {
    let filter = format!("Name {operator} '{name}'");
    let url = "/v1/Projects".to_string();

    let data: ResponseListV1<Project> = fetch(url, [Param::Filter(filter)]).await?;

    Ok(data.items)
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
