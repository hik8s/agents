use thiserror::Error;

use crate::env::EnvError;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Environment variable error: {0}")]
    EnvError(#[from] EnvError),
    #[error("Request error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
