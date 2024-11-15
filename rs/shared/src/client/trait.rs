use super::{Hik8sClient, Hik8sClientError};

pub trait Client {
    fn send_multipart_request(
        &self,
        route: &str,
        form_data: reqwest::multipart::Form,
    ) -> impl std::future::Future<Output = Result<(), Hik8sClientError>> + Send;
}

impl Client for Hik8sClient {
    fn send_multipart_request(
        &self,
        route: &str,
        form_data: reqwest::multipart::Form,
    ) -> impl std::future::Future<Output = Result<(), Hik8sClientError>> + Send {
        self.send_multipart_request(route, form_data)
    }
}
