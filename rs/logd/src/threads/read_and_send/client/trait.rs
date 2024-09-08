use super::{Hik8sClient, Hik8sClientError};

pub trait Client {
    fn new() -> Self;
    async fn send_multipart_request(
        &self,
        form_data: reqwest::multipart::Form,
    ) -> Result<(), Hik8sClientError>;
}

impl Client for Hik8sClient {
    fn new() -> Self {
        Hik8sClient::new().unwrap()
    }

    async fn send_multipart_request(
        &self,
        form_data: reqwest::multipart::Form,
    ) -> Result<(), Hik8sClientError> {
        self.send_multipart_request(form_data).await
    }
}
