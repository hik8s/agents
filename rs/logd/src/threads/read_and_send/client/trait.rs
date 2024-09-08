use super::{Hik8sClient, Hik8sClientError};

pub trait Client {
    async fn send_multipart_request(
        &self,
        form_data: reqwest::multipart::Form,
    ) -> Result<(), Hik8sClientError>;
}

impl Client for Hik8sClient {
    async fn send_multipart_request(
        &self,
        form_data: reqwest::multipart::Form,
    ) -> Result<(), Hik8sClientError> {
        self.send_multipart_request(form_data).await
    }
}
