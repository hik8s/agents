mod client;
mod error;
mod read_and_send;
mod reader;

pub use error::ReadThreadError;
pub use read_and_send::read_file_and_send_data;
