#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::mpsc;
    use std::thread;

    use super::super::reader::read_single_lines;

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
