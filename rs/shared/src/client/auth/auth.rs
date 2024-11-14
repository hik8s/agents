use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::env::get_env_var;

use super::AuthError;

#[derive(Debug, Serialize, Deserialize)]
struct Auth0TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Clone)]
pub struct Auth {
    auth0_domain: String,
    client_id: String,
    audience: String,
    token: Arc<Mutex<Option<(String, Instant)>>>,
}

impl Auth {
    pub fn new() -> Result<Self, AuthError> {
        Ok(Self {
            auth0_domain: get_env_var("AUTH0_DOMAIN")?,
            client_id: get_env_var("CLIENT_ID")?,
            audience: get_env_var("AUTH0_AUDIENCE")?,
            token: Arc::new(Mutex::new(None)),
        })
    }

    async fn fetch_auth0_token(&self) -> Result<(String, Instant), AuthError> {
        let client = Client::new();
        let client_secret = get_env_var("CLIENT_SECRET")?;

        let params = [
            ("client_id", self.client_id.clone()),
            ("client_secret", client_secret),
            ("audience", self.audience.clone()),
            ("grant_type", "client_credentials".to_string()),
        ];

        let res = client
            .post(&format!("https://{}/oauth/token", self.auth0_domain))
            .form(&params)
            .send()
            .await?;

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
        dotenv::dotenv().ok();
        // Ensure the environment variables are set to valid values before running this test
        let auth = Auth::new()?;
        let result = auth.get_auth0_token().await?;
        assert!(!result.is_empty());
        let result2 = auth.get_auth0_token().await?;
        assert_eq!(result, result2);
        Ok(())
    }
}
