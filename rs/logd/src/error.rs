use shared::tracing::TracingSetupError;
use thiserror::Error;

use crate::threads::{
    process_file_events::EventThreadError,
    read_and_send::{Hik8sClientError, ReadThreadError},
};

#[derive(Error, Debug)]
pub enum LogDaemonError {
    #[error("Tracing setup error: {0}")]
    TracingSetup(#[from] TracingSetupError),
    #[error("Event thread error: {0}")]
    EventThread(#[from] EventThreadError),
    #[error("Read thread error: {0}")]
    ReadThread(#[from] ReadThreadError),
    #[error("Task join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("Hik8s client error: {0}")]
    Hik8sClient(#[from] Hik8sClientError),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
