use reqwest::multipart::Form;
use std::sync::{Arc, Mutex};

use super::{Client, Hik8sClientError};

pub struct MockHik8sClient {
    received_data: Arc<Mutex<Vec<Form>>>,
}

#[cfg(test)]
impl MockHik8sClient {
    pub fn new(received_data: Arc<Mutex<Vec<Form>>>) -> Self {
        MockHik8sClient { received_data }
    }
}

impl Client for MockHik8sClient {
    async fn send_multipart_request(&self, form_data: Form) -> Result<(), Hik8sClientError> {
        // received data is evalued in the test
        let mut data = self.received_data.lock().unwrap();
        data.push(form_data);
        Ok(())
    }
}
