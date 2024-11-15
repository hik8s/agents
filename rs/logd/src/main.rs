use constant::LOG_PATH;
use error::LogDaemonError;
use shared::client::Hik8sClient;
use threads::process_file_events::process_file_events;
use threads::read_and_send::read_file_and_send_data;

use shared::tracing::setup_tracing;
use tokio::task::JoinHandle;
use tracing::info;

mod constant;
mod error;
mod test;
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
        info!("File events thread finished");
        Ok(())
    }));

    // Read and send thread
    let client = Hik8sClient::new()?;
    let termination_signal_clone = Arc::clone(&termination_signal);
    threads.push(tokio::spawn(async move {
        read_file_and_send_data(file_event_receiver, client, termination_signal_clone).await?;
        info!("Read and send thread finished");
        Ok(())
    }));

    // Handle thread errors
    for thread in threads {
        thread.await??;
    }

    Ok(())
}
