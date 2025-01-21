use thiserror::Error;

use crate::env::EnvError;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] EnvError),
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
