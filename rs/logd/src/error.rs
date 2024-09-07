use thiserror::Error;

use crate::{
    threads::{listen_file_event::EventThreadError, read_and_send::ReadThreadError},
    util::tracing::TracingSetupError,
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
}
