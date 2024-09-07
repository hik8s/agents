use std::env::VarError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("VarError: {0}, variable: {1}")]
    EnvVar(VarError, String),
}
