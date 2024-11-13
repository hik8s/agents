use crate::constant::HIK8S_ROUTE_LOG;
use reqwest::header::AUTHORIZATION;
use reqwest::{multipart::Form, Client};
use shared::env::get_env_var;

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
    pub fn get_uri(&self) -> String {
        format!("https://{}/{}", self.host, HIK8S_ROUTE_LOG)
    }
    pub async fn send_multipart_request(&self, form: Form) -> Result<(), Hik8sClientError> {
        let token = self.auth.get_auth0_token().await.unwrap();

        self.client
            .post(&self.get_uri())
            .multipart(form)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
