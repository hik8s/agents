#[cfg(test)]
mod integration_tests {

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    use crate::threads::process_file_events::{process_file_events, EventThreadError};
    use crate::util::test::test_util::create_test_file;
    use crate::util::tracing::setup_tracing;

    #[test]
    fn test_process_file_events_picks_up_new_file() -> Result<(), EventThreadError> {
        setup_tracing()?;
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        // Create a new file in the LOG_PATH directory
        let file1_path = create_test_file(&temp_path, "file1")?;

        let termination_signal = Arc::new(AtomicBool::new(false));
        let termination_signal_clone = Arc::clone(&termination_signal);

        // Create a channel for communication
        let (sender, receiver) = mpsc::channel();

        // Spawn a thread to run the process_file_events function
        let temp_path_clone = temp_path.clone();
        let handle = thread::spawn(move || {
            process_file_events(&temp_path_clone, sender, termination_signal_clone)
                .expect("Failed to process file events");
        });

        // Create a new file in the LOG_PATH directory
        let file2_path = create_test_file(&temp_path, "file2")?;

        // Collect received paths
        let mut received_paths = Vec::new();
        let timeout = Duration::from_millis(100);
        let start = std::time::Instant::now();

        // Loop to receive multiple events
        while start.elapsed() < timeout {
            match receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(paths) => {
                    received_paths.extend(paths);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(e) => panic!("Error receiving events: {:?}", e),
            }
        }

        // Check if the new file paths are in the received paths
        assert!(received_paths.contains(&file1_path));
        assert!(received_paths.contains(&file2_path));

        // Signal the thread to stop
        termination_signal.store(true, Ordering::SeqCst);

        handle.join().expect("Failed to join thread");
        Ok(())
    }
}
