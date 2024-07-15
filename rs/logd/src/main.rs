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
    
    let (event_tx, event_rx) = mpsc::channel();

    // Spawn a new thread to read events
    std::thread::spawn(move || {
        loop {
            let events = inotify.read_events_blocking(&mut event_buffer)
                .expect("Error while reading events");
            tracing::debug!("CLOSE_WRITE event detected");
            let mut has_event = false;
            for event in events {
                if event.mask.contains(EventMask::Q_OVERFLOW) {
                    tracing::warn!("Event queue overflowed; some events may have been lost");
                }
                tracing::debug!("{:?}", event);
                has_event = true;
                // Send the first event to the main thread and break the loop
            }
            event_tx.send(has_event).expect("Failed to send event");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
    
    // Main thread
    let mut counter = 0;
    loop {
        // Receive the event from the other thread
        let has_event = event_rx.recv().expect("Failed to receive event");
    
        // Read new entries
        if has_event {
            tracing::debug!("Reading new entries");
            let (event_tx, event_rx) = mpsc::channel();
            if let Err(e) = reader::read_chunk(&mut reader, 1048576, event_tx) {
                tracing::error!("Failed to read lines: {}", e);
            }
            for line in event_rx {
                counter += 1;
                if counter % 10000 == 0 {
                    tracing::info!("count: {} | {}", counter, line);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
