use bytes::Bytes;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
};
use tokio::sync::mpsc::UnboundedSender;

use super::ReaderError;

#[cfg(test)]
use std::sync::mpsc::Sender;

#[cfg(test)]
pub fn read_single_lines(
    reader: &mut impl BufRead,
    line: &mut String,
    tx: Sender<String>,
) -> Result<(), ReaderError> {
    loop {
        line.clear();
        match reader.read_line(line) {
            Ok(0) => break, // EOF reached
            Ok(_) => {
                tx.send(line.trim_end().to_string())?;
            }
            Err(error) => return Err(ReaderError::Io(error)),
        }
    }
    Ok(())
}

pub fn get_reader(mut file: File, position: u64) -> Result<BufReader<File>, std::io::Error> {
    file.seek(std::io::SeekFrom::Start(position))?;
    Ok(BufReader::new(file))
}

pub fn read_chunk(
    reader: &mut impl BufRead,
    batch_size: usize,
    tx: UnboundedSender<Result<Bytes, hyper::Error>>,
) -> Result<(), ReaderError> {
    let mut buffer = String::with_capacity(batch_size);
    let mut line = String::new();
    loop {
        buffer.clear();
        let mut bytes_read = 0;
        while bytes_read < batch_size {
            line.clear();
            // if line exceeds batch_size, buffer will grow larger than batch_size
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF reached
                Ok(n) => {
                    bytes_read += n;
                    buffer.push_str(&line);
                }
                Err(error) => return Err(ReaderError::Io(error)),
            }
        }
        if bytes_read == 0 {
            break;
        }
        tx.send(Ok(Bytes::copy_from_slice(buffer.as_bytes())))?;
    }
    Ok(())
}
