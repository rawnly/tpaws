use std::env::VarError;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ApiError {
    #[error("Assignable #{0} not found")]
    AssignableNotFound(String),

    #[error("{0} {1}")]
    HTTP(usize, String),

    #[error("Error in the request: {0}")]
    GenericError(String),

    #[error("Error parsing json: {0}")]
    Json(String),

    #[error("Failed to parse url")]
    UrlParsing,

    #[error("Unable to extract token: {source}")]
    TokenNotFound {
        #[from]
        source: VarError,
    },
}
