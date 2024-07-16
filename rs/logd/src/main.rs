use inotify::{EventMask, Inotify, WatchMask};
use std::fs::File;
use std::io;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod reader;
use bytes::Bytes;
use reqwest::multipart::{Form, Part};
use reqwest::Body;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::Seek;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
const PATH: &str = "/var/log/pods";

fn add_watches(
    inotify: &mut Inotify,
    path: &Path,
    map: &mut HashMap<i32, PathBuf>,
    tx: &mpsc::Sender<HashSet<PathBuf>>,
) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            tracing::info!("Adding watch for {:?}", path);
            add_watches(inotify, &path, map, tx)?;
        }
    }

    let watch = inotify
        .watches()
        .add(
            path,
            WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::CLOSE_WRITE,
        )
        .expect("Failed to add a watch");

    map.insert(watch.get_watch_descriptor_id(), path.to_path_buf());

    // Send the path of the file that was added
    if path.is_file() {
        let mut paths = HashSet::new();
        paths.insert(path.to_path_buf());
        tx.send(paths).expect("Failed to send path");
    }

    Ok(())
}
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::info!("Starting logd...");
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| {
        tracing::info!("BASE_URL not set. Using default: http://host.docker.internal:8000");
        String::from("http://host.docker.internal:8000")
    });
    // Create the directory if it does not exist
    // TODO: move this to Dockerfile
    match std::fs::create_dir_all(PATH) {
        Ok(_) => tracing::info!("Created directory: {}", PATH),
        Err(error) => tracing::error!("Failed to create directory: {}. Error: {}", PATH, error),
    }

    let mut inotify = Inotify::init().expect("Error while initializing inotify instance");

    // event_buffer for reading close write events
    // fits 25.6 events (40 bytes per event)
    let mut event_buffer = [0; 65536];
    let (event_tx, event_rx) = mpsc::channel();

    // Add a watch for each file in the directory
    let mut wd_to_path = HashMap::new();
    add_watches(&mut inotify, Path::new(PATH), &mut wd_to_path, &event_tx)
        .expect("Failed to add directory watch");

    // Spawn a new thread to read events
    std::thread::spawn(move || {
        loop {
            let events = inotify
                .read_events_blocking(&mut event_buffer)
                .expect("Error while reading events");

            let mut paths = HashSet::new();
            for event in events {
                if event.mask.contains(EventMask::Q_OVERFLOW) {
                    tracing::warn!("Event queue overflowed; some events may have been lost");
                }
                if event.mask.contains(EventMask::CLOSE_WRITE)
                    || event.mask.contains(EventMask::MODIFY)
                {
                    if let Some(name) = event.name {
                        if let Some(dir_path) = wd_to_path.get(&event.wd.get_watch_descriptor_id())
                        {
                            let path = dir_path.join(name);
                            paths.insert(path);
                        }
                    }
                }
                if event.mask.contains(EventMask::CREATE) {
                    if let Some(name) = event.name {
                        if let Some(dir_path) = wd_to_path.get(&event.wd.get_watch_descriptor_id())
                        {
                            let path = dir_path.join(name);
                            if path.is_dir() {
                                add_watches(&mut inotify, &path, &mut wd_to_path, &event_tx)
                                    .expect("Failed to add directory watch");
                            }
                        }
                    }
                }
            }
            // Convert the HashSet into a Vec and send it over the channel
            event_tx.send(paths).expect("Failed to send event");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    // Main thread
    let file_positions: Arc<Mutex<HashMap<PathBuf, u64>>> = Arc::new(Mutex::new(HashMap::new()));
    let client = reqwest::Client::new();
    loop {
        let endpoint = format!("{}/class", base_url);
        let mut unique_paths = HashSet::new();
        while let Ok(paths_set) = event_rx.try_recv() {
            for path in paths_set {
                unique_paths.insert(path);
            }
        }
        let unique_paths_vec: Vec<PathBuf> = unique_paths.into_iter().collect();
        // TODO: refactor to use one file_position not the entire hashmap
        let file_positions_clone = Arc::clone(&file_positions);
        let client_clone = client.clone();
        tokio::spawn(async move {
            for path in unique_paths_vec {
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

                let mut file = match File::open(&path) {
                    Ok(file) => file,
                    Err(e) => {
                        tracing::error!("Failed to open file {}: {}", path.display(), e);
                        continue;
                    }
                };
                let position = {
                    let mut file_positions_lock =
                        file_positions_clone.lock().expect("Failed to lock mutex");
                    *file_positions_lock.entry(path.clone()).or_insert(0)
                };
                file.seek(std::io::SeekFrom::Start(position))
                    .expect("Failed to seek to position");

                let mut reader = io::BufReader::new(file);

                // Read new entries
                if let Err(e) = reader::read_chunk(&mut reader, 1048576, tx) {
                    tracing::error!("Failed to read lines from file {}: {}", path.display(), e);
                }
                // update file_position
                {
                    let mut file_positions_lock =
                        file_positions_clone.lock().expect("Failed to lock mutex");
                    *file_positions_lock.entry(path.clone()).or_insert(0) = reader
                        .stream_position()
                        .expect("Failed to get current position");
                }
                let parent_path = path.parent().unwrap().to_str().unwrap();
                let file_name = path.file_name().unwrap().to_str().unwrap();

                let metadata = serde_json::json!({
                    "path": parent_path,
                    "file": file_name
                });
                tracing::debug!("Sending lines with metadata: {}", metadata.to_string());
                let rx_stream = UnboundedReceiverStream::new(rx);
                let stream =
                    rx_stream.map(|item| Ok::<_, hyper::Error>(Bytes::copy_from_slice(&item)));

                let form = Form::new()
                    .part(
                        "metadata",
                        Part::text(metadata.to_string())
                            .mime_str("application/json")
                            .unwrap(),
                    )
                    .part(
                        "stream",
                        Part::stream(Body::wrap_stream(stream))
                            .mime_str("application/octet-stream")
                            .unwrap(),
                    );

                let res = client_clone.post(&endpoint).multipart(form).send().await;
                // TODO: retry mechanism
                match res {
                    Ok(response) => {
                        if response.status().is_success() {
                            tracing::debug!("Lines sent successfully");
                        } else {
                            let status = response.status();
                            let text = response
                                .text()
                                .await
                                .unwrap_or_else(|_| String::from("Failed to read response text"));
                            tracing::error!(
                                "Failed to send lines, status: {}, response: {}",
                                status,
                                text
                            );
                        }
                    }
                    Err(error) => match error.status() {
                        Some(status_code) => {
                            tracing::error!(
                                "Failed to send lines, status code: {}, error: {}",
                                status_code,
                                error
                            );
                        }
                        None => {
                            tracing::error!("Failed to send lines, error: {}", error);
                        }
                    },
                }
            }
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
