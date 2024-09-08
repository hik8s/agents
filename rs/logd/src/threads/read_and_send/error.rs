use std::io;
use thiserror::Error;

use super::client::Hik8sClientError;

#[derive(Error, Debug)]
pub enum ReadThreadError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Hik8s client error: {0}")]
    Hik8sClient(#[from] Hik8sClientError),
}
