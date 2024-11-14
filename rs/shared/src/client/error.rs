use thiserror::Error;

use crate::env::EnvError;

use super::auth::AuthError;

#[derive(Error, Debug)]
pub enum Hik8sClientError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Environment variable error: {0}")]
    EnvError(#[from] EnvError),
    #[error("Auth error: {0}")]
    AuthError(#[from] AuthError),
}
