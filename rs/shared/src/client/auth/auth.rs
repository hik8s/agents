use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::env::{get_env_audience, get_env_var};

use super::AuthError;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Auth0TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Clone)]
pub struct Auth {
    client: Client,
    domain: String,
    client_id: String,
    audience: Vec<String>,
    token: Arc<Mutex<Option<(String, Instant)>>>,
}

impl Auth {
    pub fn new() -> Result<Self, AuthError> {
        let client = Client::builder().use_rustls_tls().build()?;
        Ok(Self {
            client,
            domain: get_env_var("AUTH_DOMAIN")?,
            client_id: get_env_var("CLIENT_ID")?,
            audience: get_env_audience()?,
            token: Arc::new(Mutex::new(None)),
        })
    }

    async fn fetch_auth0_token(&self) -> Result<(String, Instant), AuthError> {
        let mut params = vec![("grant_type", "client_credentials".to_string())];

        for audience in &self.audience {
            params.push(("audience", audience.clone()));
        }

        let res = self
            .client
            .post(format!("https://{}/oauth2/token", self.domain))
            .basic_auth(&self.client_id, Some(get_env_var("CLIENT_SECRET")?))
            .form(&params)
            .send()
            .await?
            .error_for_status()?;

        let token_response: Auth0TokenResponse = res.json().await?;
        let expiration_time = Instant::now() + Duration::from_secs(token_response.expires_in);
        Ok((token_response.access_token, expiration_time))
    }

    pub async fn get_auth0_token(&self) -> Result<String, AuthError> {
        let mut token_guard = self.token.lock().await;
        if let Some((ref token, ref expiration_time)) = *token_guard {
            if Instant::now() < *expiration_time {
                return Ok(token.clone());
            }
        }

        let (new_token, new_expiration_time) = self.fetch_auth0_token().await?;
        *token_guard = Some((new_token.clone(), new_expiration_time));
        Ok(new_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_auth0_token_real() -> Result<(), AuthError> {
        // Ensure the environment variables are set to valid values before running this test
        let auth = Auth::new()?;
        let result = auth.get_auth0_token().await?;
        assert!(!result.is_empty());
        let result2 = auth.get_auth0_token().await?;
        assert_eq!(result, result2);
        Ok(())
    }
}
