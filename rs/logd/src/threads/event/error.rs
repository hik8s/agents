use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventThreadError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}
