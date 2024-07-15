use inotify::{Inotify, WatchMask, EventMask};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::fs::File;
use std::io;
mod reader;
use std::sync::mpsc;

const TEST_FILE_PATH: &str = "/tmp/inotify-rs-test-file";
fn main() {
    tracing_subscriber::registry()
        .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
    tracing::info!("Starting logd...");

    let mut inotify = Inotify::init()
        .expect("Error while initializing inotify instance");

    // Watch for modify and close events.
    inotify
        .watches()
        .add(
            TEST_FILE_PATH,
            WatchMask::CLOSE_WRITE, // could also use WatchMask::MODIFY
        )
        .expect("Failed to add file watch");


    // event_buffer for reading close write events
    // fits 25.6 events (40 bytes per event)
    let mut event_buffer = [0; 1024];
    let file = File::open(TEST_FILE_PATH).expect("Failed to open log file");
    let mut reader = io::BufReader::new(file);
    
    loop {
        let events = inotify.read_events_blocking(&mut event_buffer)
            .expect("Error while reading events");
        tracing::info!("Read events");

        let mut has_event = false;
        for event in events {
            has_event = true;
            if event.mask.contains(EventMask::Q_OVERFLOW) {
                tracing::warn!("Event queue overflowed; some events may have been lost");
            }
        }

        // Read new entries
        if has_event {
            let (tx, rx) = mpsc::channel();
            if let Err(e) = reader::read_chunk(&mut reader, 1048576, tx) {
                tracing::error!("Failed to read lines: {}", e);
            }
            for line in rx {
                tracing::debug!("{}", line);
            }
        }
    }
}
