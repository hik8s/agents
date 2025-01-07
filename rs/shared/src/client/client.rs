use crate::env::get_env_var;
use reqwest::header::AUTHORIZATION;
use reqwest::{multipart::Form, Client};
use serde::Serialize;

use super::auth::Auth;
use super::Hik8sClientError;

#[derive(Clone)]
pub struct Hik8sClient {
    pub client: Client,
    pub client_with_middleware: ClientWithMiddleware,
    insecure: bool,
    host: String,
    port: String,
    auth: Auth,
}
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

impl Hik8sClient {
    pub fn new(insecure: bool) -> Result<Self, Hik8sClientError> {
        let host = get_env_var("HIK8S_HOST")?;
        let port = get_env_var("HIK8S_PORT")?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(10);
        let client = Client::builder().use_rustls_tls().build()?;
        let client_with_middleware = ClientBuilder::new(client.clone())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        let auth = Auth::new()?;
        Ok(Self {
            client,
            client_with_middleware,
            insecure,
            host,
            port,
            auth,
        })
    }
    pub fn get_uri(&self, route: &str) -> String {
        let protocol = if self.insecure { "http" } else { "https" };
        format!("{}://{}:{}/{route}", protocol, self.host, self.port)
    }
    pub async fn send_multipart_request(
        &self,
        route: &str,
        form: Form,
    ) -> Result<(), Hik8sClientError> {
        let token = self.auth.get_auth0_token().await.unwrap();

        self.client
            .post(self.get_uri(route))
            .multipart(form)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
    pub async fn send_request(
        &self,
        route: &str,
        json: &serde_json::Value,
    ) -> Result<(), Hik8sClientError> {
        let token = self.auth.get_auth0_token().await.unwrap();

        self.client_with_middleware
            .post(self.get_uri(route))
            .json(json)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
