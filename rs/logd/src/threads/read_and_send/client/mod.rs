mod auth;
mod client;
mod error;
mod form;
pub mod mock;
mod r#trait;

pub use client::Hik8sClient;
pub use error::Hik8sClientError;
pub use form::create_form_data;
pub use r#trait::Client;

#[cfg(test)]
pub use mock::MockHik8sClient;
