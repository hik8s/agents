use inotify::{Inotify, WatchMask, EventMask};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::fs::File;
use std::io;
mod reader;
use std::sync::mpsc;
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use std::path::PathBuf;
use bytes::Bytes;
use std::convert::Infallible;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

const PATH: &str = "/var/log/pods";
// /var/log/pods/<pod>/<container>/<0.log>

fn add_watches(inotify: &mut Inotify, path: &Path, map: &mut HashMap<i32, PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            tracing::info!("Adding watch for {:?}", path);
            add_watches(inotify, &path, map)?;
        }
    }

    let watch = inotify
        .watches().add(
            path,
            WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::CLOSE_WRITE,
        )
        .expect("Failed to add a watch");

    map.insert(watch.get_watch_descriptor_id(), path.to_path_buf());

    Ok(())
}
#[tokio::main]
async fn main() {
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

    // Add a watch for each file in the directory
    let mut wd_to_path = HashMap::new();
    add_watches(&mut inotify, Path::new(PATH), &mut wd_to_path)
        .expect("Failed to add directory watch");

    // event_buffer for reading close write events
    // fits 25.6 events (40 bytes per event)    
    let mut event_buffer = [0; 1024];
    let (event_tx, event_rx) = mpsc::channel();

    // Spawn a new thread to read events
    std::thread::spawn(move || {
        loop {
            let events = inotify.read_events_blocking(&mut event_buffer)
                .expect("Error while reading events");

            for event in events {
                if event.mask.contains(EventMask::Q_OVERFLOW) {
                    tracing::warn!("Event queue overflowed; some events may have been lost");
                }
                if event.mask.contains(EventMask::CLOSE_WRITE) {
                    tracing::debug!("CLOSE_WRITE event detected");
                    if let Some(name) = event.name {
                        tracing::debug!("name {:?}", name);
                        if let Some(dir_path) = wd_to_path.get(&event.wd.get_watch_descriptor_id()) {
                            let path = dir_path.join(name);
                            tracing::debug!("path {}", path.to_str().unwrap());
                            // Send the path of the changed file to the main thread
                            event_tx.send(path).expect("Failed to send event");
                        }
                    }
                }
                if event.mask.contains(EventMask::CREATE) {
                    tracing::debug!("CREATE event detected");
                    if let Some(name) = event.name {
                        if let Some(dir_path) = wd_to_path.get(&event.wd.get_watch_descriptor_id()) {
                            let path = dir_path.join(name);
                            if path.is_dir() {
                                tracing::debug!("Adding watch for new directory: {:?}", path);
                                add_watches(&mut inotify, &path, &mut wd_to_path)
                                    .expect("Failed to add directory watch");
                            }
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
    
    // Main thread
    use std::io::Seek;
    use std::sync::mpsc::TryRecvError;
    
    let mut file_positions: HashMap<PathBuf, u64> = HashMap::new();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    std::thread::spawn(move || {
        loop {
            // Try to receive the event from the other thread
            match event_rx.try_recv() {
                Ok(path) => {
                    tracing::debug!("Event for file: {:?}", path);

                    let mut file = File::open(&path).expect("Failed to open log file");
                    let position = file_positions.entry(path.clone()).or_insert(0);
                    file.seek(std::io::SeekFrom::Start(*position)).expect("Failed to seek to position");

                    let mut reader = io::BufReader::new(file);

                    // Read new entries
                    if let Err(e) = reader::read_chunk(&mut reader, 1048576, tx.clone()) {
                        tracing::error!("Failed to read lines: {}", e);
                    }
                    // update file_position
                    *position = reader.stream_position().expect("Failed to get current position");
                },
                Err(TryRecvError::Empty) => {
                    // No event to receive
                },
                Err(TryRecvError::Disconnected) => {
                    tracing::error!("Channel has been disconnected");
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
    tracing::info!("sending request...");
    let client = reqwest::Client::new();

    let rx_stream = UnboundedReceiverStream::new(rx);
    let stream = rx_stream.map(|item| Ok::<_, Infallible>(Bytes::from(item.into_bytes())));

    let res = client.post("http://host.docker.internal:8000/lines")
        .body(reqwest::Body::wrap_stream(stream))
        .send()
        .await;

    match res {
        Ok(_) => tracing::info!("Lines sent successfully"),
        Err(e) => tracing::error!("Failed to send lines: {}", e),
    }
    tracing::info!("Main thread done...");
}
