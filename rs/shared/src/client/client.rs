use crate::env::get_env_var;
use reqwest::header::AUTHORIZATION;
use reqwest::{multipart::Form, Client};

use super::auth::Auth;
use super::Hik8sClientError;

pub struct Hik8sClient {
    pub client: Client,
    pub host: String,
    auth: Auth,
}

impl Hik8sClient {
    pub fn new() -> Result<Self, Hik8sClientError> {
        let host = get_env_var("HIK8S_HOST")?;
        let client = Client::builder().use_rustls_tls().build()?;
        let auth = Auth::new()?;
        Ok(Self { client, host, auth })
    }
    pub fn get_uri(&self, route: &str) -> String {
        format!("https://{}/{}", self.host, route)
    }
    pub async fn send_multipart_request(
        &self,
        route: &str,
        form: Form,
    ) -> Result<(), Hik8sClientError> {
        let token = self.auth.get_auth0_token().await.unwrap();

        self.client
            .post(&self.get_uri(route))
            .multipart(form)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
