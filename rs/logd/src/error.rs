use thiserror::Error;

use crate::{
    client::Hik8sClientError,
    threads::{event::EventThreadError, read::ReadThreadError},
};

#[derive(Error, Debug)]
pub enum LogDaemonError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Event thread error: {0}")]
    EventThreadError(#[from] EventThreadError),
    #[error("Read thread error: {0}")]
    ReadThreadError(#[from] ReadThreadError),
    #[error("Hik8s client error: {0}")]
    Hik8sClientError(#[from] Hik8sClientError),
    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}
