mod client;
mod error;
mod read;
mod reader;

pub use error::ReadThreadError;
pub use read::read_file_and_send_data;
