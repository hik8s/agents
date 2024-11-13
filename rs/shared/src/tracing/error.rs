use thiserror::Error;
use tracing::subscriber::SetGlobalDefaultError;

use crate::env::EnvError;

#[derive(Error, Debug)]
pub enum TracingSetupError {
    #[error("Environment variable error: {0}")]
    EnvError(#[from] EnvError),
    #[error("Subscriber error: {0}")]
    SetGlobalDefaultError(#[from] SetGlobalDefaultError),
}
