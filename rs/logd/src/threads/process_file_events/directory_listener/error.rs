use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirectoryListenerError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to send path: {0}")]
    Send(#[from] std::sync::mpsc::SendError<std::collections::HashSet<std::path::PathBuf>>),
}
