use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadThreadError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}
