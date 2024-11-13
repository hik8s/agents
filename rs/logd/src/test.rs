#[cfg(test)]
mod integration_tests {
    use std::fs::create_dir;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::task::JoinHandle;
    use tracing::info;

    use crate::error::LogDaemonError;
    use crate::threads::process_file_events::process_file_events;
    use crate::threads::read_and_send::read_file_and_send_data;
    use crate::threads::read_and_send::MockHik8sClient;
    use crate::util::test::test_util::create_test_file;
    use shared::tracing::setup_tracing;

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_main_with_mock_client() -> Result<(), LogDaemonError> {
        setup_tracing()?;
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        // Track threads
        let mut threads: Vec<JoinHandle<Result<(), LogDaemonError>>> = Vec::new();

        let sig_term1 = Arc::new(AtomicBool::new(false));
        let sig_term1_clone = Arc::clone(&sig_term1);

        // File events thread
        let temp_path_clone = temp_path.clone();
        let (file_event_sender, file_event_receiver) = mpsc::channel();
        threads.push(tokio::spawn(async move {
            process_file_events(&temp_path_clone, file_event_sender, sig_term1_clone)?;
            info!("File events thread finished");
            Ok(())
        }));

        // Mock client
        let received_data = Arc::new(Mutex::new(Vec::new()));
        let mock_client = MockHik8sClient::new(Arc::clone(&received_data));

        // Read and send thread
        let sig_term2 = Arc::new(AtomicBool::new(false));
        let sig_term2_clone = Arc::clone(&sig_term2);
        threads.push(tokio::spawn(async move {
            read_file_and_send_data(file_event_receiver, mock_client, sig_term2_clone).await?;
            info!("Read and send thread finished");
            Ok(())
        }));

        // tokio::time::sleep(Duration::from_millis(100)).await;

        let subdir_path = temp_path.join("subdir");
        create_dir(&subdir_path)?;
        create_test_file(&temp_path, "file1")?;
        create_test_file(&subdir_path, "file2")?;
        create_test_file(&temp_path, "file3")?;

        tokio::time::sleep(Duration::from_millis(10)).await;
        info!("Terminating threads..");
        sig_term1.store(true, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(10)).await;
        sig_term2.store(true, Ordering::SeqCst);

        // Handle thread errors
        for thread in threads {
            thread.await??;
        }
        info!("Threads finished");

        let data: std::sync::MutexGuard<'_, Vec<reqwest::multipart::Form>> =
            received_data.lock().unwrap();
        assert!(!data.is_empty(), "No data received by the mock client");
        assert_eq!(data.len(), 3);

        Ok(())
    }
}
