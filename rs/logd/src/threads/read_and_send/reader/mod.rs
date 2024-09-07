mod error;
mod reader;
mod test;

pub use error::ReaderError;
pub use reader::{get_reader, read_chunk};
