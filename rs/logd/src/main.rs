use error::LogDaemonError;
use threads::event::process_file_events;
use threads::read::read_file_and_send_data;

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

    // Spawn a new thread to read events
    let mut threads = Vec::new();

    let (file_event_sender, file_event_receiver) = mpsc::channel();
    threads.push(tokio::spawn(async move {
        process_file_events(file_event_sender)?;
        Ok::<(), LogDaemonError>(())
    }));

    threads.push(tokio::spawn(async move {
        read_file_and_send_data(file_event_receiver).await?;
        Ok::<(), LogDaemonError>(())
    }));

    for thread in threads {
        thread.await??;
    }

    Ok(())
}
