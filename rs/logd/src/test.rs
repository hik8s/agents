#[cfg(test)]
mod integration_tests {
    use shared::client::Hik8sClient;
    use std::env;
    use std::fs::create_dir;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::time::{Duration, Instant};
    use tempfile::tempdir;
    use tokio::task::JoinHandle;
    use tracing::debug;

    use crate::constant::HIK8S_ROUTE_LOG;
    use crate::error::LogDaemonError;
    use crate::threads::process_file_events::process_file_events;
    use crate::threads::read_and_send::read_file_and_send_data;
    use crate::util::test::test_util::create_test_file;
    use shared::tracing::setup_tracing;

    use httpmock::Method::POST;
    use httpmock::MockServer;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_main_with_mock_client() -> Result<(), LogDaemonError> {
        setup_tracing()?;
        // Track threads
        let mut threads: Vec<JoinHandle<Result<(), LogDaemonError>>> = Vec::new();

        // Start mock server
        let server = MockServer::start_async().await;
        let server_port = server.port().to_string();
        let mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path(format!("/{HIK8S_ROUTE_LOG}"))
                    .is_true(|req| {
                        req.body()
                            .to_maybe_lossy_str()
                            .contains("This is the first line of")
                    });
                then.status(200);
            })
            .await;

        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        let sig_term = Arc::new(AtomicBool::new(false));
        let sig_term_clone = sig_term.clone();

        // File events thread
        let temp_path_clone = temp_path.clone();
        let (file_event_sender, file_event_receiver) = mpsc::channel();
        threads.push(tokio::spawn(async move {
            process_file_events(&temp_path_clone, file_event_sender, sig_term_clone)?;
            debug!("File events thread finished");
            Ok(())
        }));

        // Mock client
        dotenv::dotenv().ok();
        env::set_var("HIK8S_PORT", server_port);
        let client = Hik8sClient::new(true)?;

        // Read and send thread
        let sig_term_clone = sig_term.clone();
        threads.push(tokio::spawn(async move {
            read_file_and_send_data(file_event_receiver, client, sig_term_clone).await?;
            debug!("Read and send thread finished");
            Ok(())
        }));

        let subdir_path = temp_path.join("subdir");
        create_dir(&subdir_path).unwrap();
        create_test_file(&temp_path, "file1").unwrap();
        create_test_file(&subdir_path, "file2").unwrap();
        create_test_file(&temp_path, "file3").unwrap();
        let expected_hits = 3;

        let start_time = Instant::now();
        let timeout_duration = Duration::from_secs(1);
        while mock.calls() < expected_hits {
            if start_time.elapsed() >= timeout_duration {
                sig_term.store(true, Ordering::SeqCst);
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        assert_eq!(
            mock.calls(),
            expected_hits,
            "Not all requests were received"
        );

        // Handle thread errors
        debug!("Terminating threads..");
        sig_term.store(true, Ordering::SeqCst);
        for thread in threads {
            thread.await??;
        }
        debug!("Threads finished");

        Ok(())
    }
}
