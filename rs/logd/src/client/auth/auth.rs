use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::util::env::get_env_var;

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
}

impl Auth {
    pub fn new() -> Result<Self, AuthError> {
        Ok(Self {
            auth0_domain: get_env_var("AUTH0_DOMAIN")?,
            client_id: get_env_var("CLIENT_ID")?,
            audience: get_env_var("AUTH0_AUDIENCE")?,
        })
    }

    pub async fn get_auth0_token(&self) -> Result<String, AuthError> {
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
        Ok(token_response.access_token)
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
        Ok(())
    }
}
