use std::env::var;

use super::EnvError;

pub fn get_env_var(key: &str) -> Result<String, EnvError> {
    var(key).map_err(|e| EnvError::EnvVar(e, key.to_owned()))
}
