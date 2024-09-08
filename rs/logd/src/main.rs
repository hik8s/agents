use constant::LOG_PATH;
use error::LogDaemonError;
use threads::process_file_events::process_file_events;
use threads::read_and_send::{read_file_and_send_data, Hik8sClient};

use tokio::task::JoinHandle;
use tracing::info;
use util::tracing::setup_tracing;

mod constant;
mod error;
mod threads;
mod util;

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

#[tokio::main]
async fn main() -> Result<(), LogDaemonError> {
    setup_tracing()?;
    info!("Starting logd...");

    // Track threads
    let mut threads: Vec<JoinHandle<Result<(), LogDaemonError>>> = Vec::new();

    let termination_signal = Arc::new(AtomicBool::new(false));
    let termination_signal_clone = Arc::clone(&termination_signal);

    // File events thread
    let (file_event_sender, file_event_receiver) = mpsc::channel();
    threads.push(tokio::spawn(async move {
        process_file_events(
            Path::new(LOG_PATH),
            file_event_sender,
            termination_signal_clone,
        )?;
        Ok(())
    }));

    // Read and send thread
    let client = Hik8sClient::new()?;
    threads.push(tokio::spawn(async move {
        read_file_and_send_data(file_event_receiver, client).await?;
        Ok(())
    }));

    // Handle thread errors
    for thread in threads {
        thread.await??;
    }

    Ok(())
}
