use std::{io, sync::mpsc::RecvTimeoutError};
use thiserror::Error;

use shared::{
    client::{FormDataError, Hik8sClientError},
    tracing::TracingSetupError,
};

use super::reader::ReaderError;

#[derive(Error, Debug)]
pub enum ReadThreadError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Hik8s client error: {0}")]
    Hik8sClient(#[from] Hik8sClientError),
    #[error("Tracing setup error: {0}")]
    TracingSetup(#[from] TracingSetupError),
    #[error("Channel receive timeout: {0}")]
    RecvTimeout(#[from] RecvTimeoutError),
    #[error("Form data error: {0}")]
    FormData(#[from] FormDataError),
    #[error("Reader error: {0}")]
    Reader(#[from] ReaderError),
}
