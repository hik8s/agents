use error::LogDaemonError;
use threads::file_event::process_file_events;
use threads::read_and_send::read_file_and_send_data;

use tokio::task::JoinHandle;
use tracing::info;
use util::tracing::setup_tracing;

mod client;
mod constant;
mod error;
mod threads;
mod util;

use std::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), LogDaemonError> {
    setup_tracing();
    info!("Starting logd...");

    // Track threads
    let mut threads: Vec<JoinHandle<Result<(), LogDaemonError>>> = Vec::new();

    // File events thread
    let (file_event_sender, file_event_receiver) = mpsc::channel();
    threads.push(tokio::spawn(async move {
        process_file_events(file_event_sender)?;
        Ok(())
    }));

    // Read and send thread
    threads.push(tokio::spawn(async move {
        read_file_and_send_data(file_event_receiver).await?;
        Ok(())
    }));

    // Handle thread errors
    for thread in threads {
        thread.await??;
    }

    Ok(())
}
