#[cfg(test)]
mod integration_tests {
    use std::collections::HashSet;
    use std::sync::atomic::AtomicBool;
    use std::sync::{mpsc, Arc, Mutex};
    use tempfile::tempdir;
    use tracing::info;

    use crate::threads::read_and_send::client::MockHik8sClient;
    use crate::threads::read_and_send::{read_file_and_send_data, ReadThreadError};
    use crate::util::test::test_util::create_test_file;
    use shared::tracing::setup_tracing;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_read_file_and_send_data() -> Result<(), ReadThreadError> {
        setup_tracing()?;

        // Create a temporary directory
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path().to_path_buf();

        // Create the first test file
        let file1_path = create_test_file(&temp_path, "file1.txt")?;
        let file2_path = create_test_file(&temp_path, "file2.txt")?;
        let file3_path = create_test_file(&temp_path, "file3.txt")?;
        let file4_path = create_test_file(&temp_path, "file4.gz")?;

        // Create a channel for communication
        let (sender, receiver) = mpsc::channel();

        // Shared state to store received data
        let received_data = Arc::new(Mutex::new(Vec::new()));
        let client = MockHik8sClient::new(Arc::clone(&received_data));

        // Spawn a thread to run the read_file_and_send_data function
        let termination_signal = Arc::new(AtomicBool::new(false));
        let termination_signal_clone = Arc::clone(&termination_signal);
        let handle = tokio::spawn(async move {
            read_file_and_send_data(receiver, client, termination_signal_clone)
                .await
                .expect("Failed to read and send data");
        });

        // Send two path with data
        let mut paths = HashSet::new();
        paths.insert(file1_path);
        paths.insert(file2_path);
        sender.send(paths).unwrap();

        // Send one path with data
        let mut paths = HashSet::new();
        paths.insert(file3_path);
        sender.send(paths).unwrap();

        // Send one path with .gz file
        let mut paths = HashSet::new();
        paths.insert(file4_path);
        sender.send(paths).unwrap();

        // Wait for the thread to process the files
        drop(sender);
        handle.await.unwrap();

        info!("Threads finished");
        // Verify that the files were read and data was sent
        let data: std::sync::MutexGuard<'_, Vec<reqwest::multipart::Form>> =
            received_data.lock().unwrap();
        assert!(!data.is_empty(), "No data received by the mock client");
        assert_eq!(data.len(), 3);
        Ok(())
    }
}
