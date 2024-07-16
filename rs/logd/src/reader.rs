use std::io::{self, BufRead};
use std::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedSender;

#[allow(unused)]
pub fn read_single_lines(
    reader: &mut impl BufRead,
    line: &mut String,
    tx: Sender<String>,
) -> io::Result<()> {
    loop {
        line.clear();
        match reader.read_line(line) {
            Ok(0) => break, // EOF reached
            Ok(_) => {
                tx.send(line.trim_end().to_string()).unwrap();
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}
pub fn read_chunk(
    reader: &mut impl BufRead,
    batch_size: usize,
    tx: UnboundedSender<Vec<u8>>,
) -> io::Result<()> {
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
                Err(error) => return Err(error),
            }
        }
        if bytes_read == 0 {
            break;
        }
        tx.send(buffer.as_bytes().to_vec()).unwrap();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn test_read_single_lines_empty() {
        let data = "";
        let mut reader = Cursor::new(data);
        let mut buffer = String::new();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            assert!(read_single_lines(&mut reader, &mut buffer, tx).is_ok());
        });
        assert!(rx.recv().is_err()); // No messages should be sent
    }

    #[test]
    fn test_read_single_lines_single_line() {
        let data = "Hello, world!\n";
        let mut reader = Cursor::new(data);
        let mut buffer = String::new();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            assert!(read_single_lines(&mut reader, &mut buffer, tx).is_ok());
        });
        assert_eq!(rx.recv().unwrap(), "Hello, world!"); // One message should be sent
        assert!(rx.recv().is_err()); // No more messages should be sent
    }

    #[test]
    fn test_read_single_lines_multiple_lines() {
        let data = "Hello, world!\nGoodbye, world!\n";
        let mut reader = Cursor::new(data);
        let mut buffer = String::new();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            assert!(read_single_lines(&mut reader, &mut buffer, tx).is_ok());
        });
        assert_eq!(rx.recv().unwrap(), "Hello, world!"); // First message
        assert_eq!(rx.recv().unwrap(), "Goodbye, world!"); // Second message
        assert!(rx.recv().is_err()); // No more messages should be sent
    }
    #[test]
    fn test_read_single_lines_continues_reading() {
        let data = "Hello, world!\nGoodbye, world!\n";
        let mut reader = Cursor::new(&data[0..14]); // Only "Hello, world!\n"
        let mut buffer = String::new();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            assert!(read_single_lines(&mut reader, &mut buffer, tx).is_ok());
        });
        assert_eq!(rx.recv().unwrap(), "Hello, world!"); // First message

        // Pass data to the function a second time
        let mut reader = Cursor::new(&data[14..]); // Only "Goodbye, world!\n"
        let mut buffer = String::new();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            assert!(read_single_lines(&mut reader, &mut buffer, tx).is_ok());
        });
        assert_eq!(rx.recv().unwrap(), "Goodbye, world!"); // Second message
        assert!(rx.recv().is_err()); // No more messages should be sent
    }
}
