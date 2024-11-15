use thiserror::Error;

use crate::env::EnvError;

use super::auth::AuthError;

#[derive(Error, Debug)]
pub enum Hik8sClientError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Reqwest middleware error: {0}")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("Environment variable error: {0}")]
    EnvError(#[from] EnvError),
    #[error("Auth error: {0}")]
    AuthError(#[from] AuthError),
    #[error("Json serialize error: {0}")]
    JsonError(#[from] serde_json::Error),
}
