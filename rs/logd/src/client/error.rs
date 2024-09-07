use thiserror::Error;

use crate::env::EnvError;

#[derive(Error, Debug)]
pub enum Hik8sClientError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Environment variable error: {0}")]
    EnvError(#[from] EnvError),
}
