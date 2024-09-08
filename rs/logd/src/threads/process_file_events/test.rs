#[cfg(test)]
mod integration_tests {

    use std::fs::File;
    use std::io::Write;

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;
    use tracing::info;

    use crate::error::LogDaemonError;
    use crate::threads::process_file_events::process_file_events;
    use crate::util::tracing::setup_tracing;

    #[test]
    fn test_process_file_events_picks_up_new_file() -> Result<(), LogDaemonError> {
        setup_tracing()?;
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        // Create a new file in the LOG_PATH directory
        let new_file1_path = temp_path.join("new_file1.txt");
        let mut file = File::create(&new_file1_path).expect("Failed to create new file");
        let text = "This is a line of text.";
        file.write_all(text.as_bytes())
            .expect("Failed to write to file");
        File::create(&new_file1_path).expect("Failed to create new file");

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
        let new_file2_path = temp_path.join("new_file2.txt");
        File::create(&new_file2_path).expect("Failed to create new file");
        let mut file = File::create(&new_file2_path).expect("Failed to create new file");
        let text = "This is a line of text.";
        file.write_all(text.as_bytes())
            .expect("Failed to write to file");

        // Collect received paths
        let mut received_paths = Vec::new();
        let timeout = Duration::from_secs(5);
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

        info!("Received paths: {:?}", received_paths);
        // Check if the new file paths are in the received paths
        assert!(received_paths.contains(&new_file1_path));
        assert!(received_paths.contains(&new_file2_path));

        // Signal the thread to stop
        termination_signal.store(true, Ordering::SeqCst);

        info!("Joining thread");
        handle.join().expect("Failed to join thread");
        Ok(())
    }
}
