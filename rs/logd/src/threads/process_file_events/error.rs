use shared::tracing::TracingSetupError;
use std::io;
use thiserror::Error;

use super::directory_listener::DirectoryListenerError;

#[derive(Error, Debug)]
pub enum EventThreadError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Directory listener error: {0}")]
    DirectoryListener(#[from] DirectoryListenerError),
    #[error("Tracing setup error: {0}")]
    TracingSetup(#[from] TracingSetupError),
}
