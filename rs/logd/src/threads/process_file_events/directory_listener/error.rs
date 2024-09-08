use std::sync::mpsc::SendError;
use std::{collections::HashSet, io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirectoryListenerError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to send path: {0}")]
    Send(#[from] SendError<HashSet<PathBuf>>),
}
