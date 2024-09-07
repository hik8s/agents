use thiserror::Error;

use crate::threads::{file_event::EventThreadError, read_and_send::ReadThreadError};

#[derive(Error, Debug)]
pub enum LogDaemonError {
    #[error("Event thread error: {0}")]
    EventThreadError(#[from] EventThreadError),
    #[error("Read thread error: {0}")]
    ReadThreadError(#[from] ReadThreadError),
    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}
