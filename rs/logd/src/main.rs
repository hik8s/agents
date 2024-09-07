use error::LogDaemonError;
use threads::event::process_inotify_events;
use threads::read::read_and_track_files;

use tracing::info;
use util::tracing::setup_tracing;
mod auth;
mod client;
mod constant;
mod env;
mod error;
mod form;
mod io;
mod reader;
mod threads;
mod util;

use std::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), LogDaemonError> {
    setup_tracing();
    info!("Starting logd...");

    // Spawn a new thread to read events
    let mut threads = Vec::new();

    let (event_sender, event_receiver) = mpsc::channel();
    threads.push(tokio::spawn(async move {
        process_inotify_events(event_sender)?;
        Ok::<(), LogDaemonError>(())
    }));

    threads.push(tokio::spawn(async move {
        read_and_track_files(event_receiver).await?;
        Ok::<(), LogDaemonError>(())
    }));

    for thread in threads {
        thread.await??;
    }

    Ok(())
}
