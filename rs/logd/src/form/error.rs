use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormDataError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
