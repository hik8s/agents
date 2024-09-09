mod client;
mod error;
mod read_and_send;
mod reader;
mod test;

pub use client::Hik8sClient;
pub use client::Hik8sClientError;
pub use error::ReadThreadError;
pub use read_and_send::read_file_and_send_data;

#[cfg(test)]
pub use client::mock::MockHik8sClient;
