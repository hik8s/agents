use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Auth0TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

pub async fn get_auth0_token() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let auth0_domain = env::var("AUTH0_DOMAIN")?;
    let client_id = env::var("CLIENT_ID")?;
    let client_secret = env::var("CLIENT_SECRET")?;
    let audience = env::var("AUTH0_AUDIENCE")?;

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("audience", audience),
        ("grant_type", "client_credentials".to_string()),
    ];

    let res = client
        .post(&format!("https://{}/oauth/token", auth0_domain))
        .form(&params)
        .send()
        .await?;

    let token_response: Auth0TokenResponse = res.json().await?;
    Ok(token_response.access_token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_auth0_token_real() {
        dotenv::dotenv().ok();
        // Ensure the environment variables are set to valid values before running this test
        let result = get_auth0_token().await;
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }
}
