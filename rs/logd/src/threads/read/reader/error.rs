use std::sync::mpsc::SendError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError as TokioSendError;

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Send error")]
    SendBytes(#[from] TokioSendError<Result<bytes::Bytes, hyper::Error>>),
    #[error("Send error")]
    SendString(#[from] SendError<String>),
}
