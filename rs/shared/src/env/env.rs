use std::env::var;

use tracing::warn;

use super::EnvError;

pub fn get_env_var(key: &str) -> Result<String, EnvError> {
    var(key).map_err(|e| EnvError::EnvVar(e, key.to_owned()))
}

pub fn get_env_var_as_vec(key: &str) -> Result<Option<Vec<String>>, EnvError> {
    let vec: Vec<String> = var(key)
        .map_err(|e| EnvError::EnvVar(e, key.to_owned()))?
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if vec.is_empty() {
        warn!("Empty env var: {}", key);
        return Ok(None);
    }
    Ok(Some(vec))
}
pub fn get_env_audience() -> Result<Vec<String>, EnvError> {
    match get_env_var_as_vec("AUTH_AUDIENCE")? {
        Some(audience) => Ok(audience),
        None => Err(EnvError::MissingAudience(
            "No audience values found in AUTH_AUDIENCE".to_string(),
        )),
    }
}
