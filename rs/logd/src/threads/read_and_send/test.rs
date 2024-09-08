#[cfg(test)]
mod integration_tests {
    use std::collections::HashSet;
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    use crate::threads::read_and_send::client::MockHik8sClient;
    use crate::threads::read_and_send::{read_file_and_send_data, ReadThreadError};
    use crate::util::test::test_util::create_test_file;
    use crate::util::tracing::setup_tracing;

    #[tokio::test]
    async fn test_read_file_and_send_data() -> Result<(), ReadThreadError> {
        setup_tracing()?;

        // Create a temporary directory
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path().to_path_buf();

        // Create the first test file
        let file1_path = create_test_file(&temp_path, "file1")?;
        let file2_path = create_test_file(&temp_path, "file2")?;
        let file3_path = create_test_file(&temp_path, "file3")?;

        // Create a channel for communication
        let (sender, receiver) = mpsc::channel();

        // Shared state to store received data
        let received_data = Arc::new(Mutex::new(Vec::new()));
        let client = MockHik8sClient::new(Arc::clone(&received_data));

        // Spawn a thread to run the read_file_and_send_data function
        let handle = tokio::spawn(async move {
            read_file_and_send_data(receiver, client)
                .await
                .expect("Failed to read and send data");
        });

        // Wait a bit to ensure the thread is running
        thread::sleep(Duration::from_millis(50));

        // Send two path with data
        let mut paths = HashSet::new();
        paths.insert(file1_path);
        paths.insert(file2_path);
        sender.send(paths).unwrap();

        // Send one path with data
        let mut paths = HashSet::new();
        paths.insert(file3_path);
        sender.send(paths).unwrap();

        // Wait for the thread to process the files
        handle.await.unwrap();

        // Verify that the files were read and data was sent
        let data: std::sync::MutexGuard<'_, Vec<reqwest::multipart::Form>> =
            received_data.lock().unwrap();
        assert!(!data.is_empty(), "No data received by the mock client");
        assert_eq!(data.len(), 3);
        Ok(())
    }
}
