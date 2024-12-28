use shared::client::{create_form_data, Client};
use std::io::Seek;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    sync::mpsc::Receiver,
};
use tracing::{debug, error, info};

use crate::constant::HIK8S_ROUTE_LOG;

use super::error::ReadThreadError;
use super::reader::{get_reader, read_chunk};

pub async fn read_file_and_send_data<C: Client>(
    event_receiver: Receiver<HashSet<PathBuf>>,
    client: C,
    termination_signal: Arc<AtomicBool>,
) -> Result<(), ReadThreadError> {
    info!("Starting read_file_and_send_data thread...");
    let mut positions: HashMap<PathBuf, u64> = HashMap::new();
    loop {
        if termination_signal.load(Ordering::SeqCst) {
            break;
        }
        match event_receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(paths) => {
                for path in paths {
                    // Read file
                    if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
                        info!("Skipping .gz file: {}", path.display());
                        continue;
                    } else {
                        debug!("Reading file: {}", path.display());
                    }

                    let file = match File::open(&path) {
                        Ok(file) => file,
                        Err(e) => {
                            tracing::error!("Failed to open file {}: {}", path.display(), e);
                            continue;
                        }
                    };

                    // Get file position
                    let position = positions.get(&path).unwrap_or(&0);

                    // Get reader at position
                    let mut reader = get_reader(file, *position).expect("Failed to get reader");

                    // Read new entries
                    let (data_sender, data_receiver) = tokio::sync::mpsc::unbounded_channel();
                    if let Err(e) = read_chunk(&mut reader, 1048576, data_sender) {
                        error!("Failed to read lines from file {}: {}", path.display(), e);
                        continue;
                    }

                    // Update file position
                    let new_position = reader.stream_position().unwrap();
                    positions.insert(path.clone(), new_position);

                    let parent_path = path.parent().unwrap().to_str().unwrap();
                    let file_name = path.file_name().unwrap().to_str().unwrap();

                    let metadata = serde_json::json!({
                        "path": parent_path,
                        "file": file_name
                    });

                    // Receiver stream
                    let stream = UnboundedReceiverStream::new(data_receiver);

                    // Form data
                    let form_data = create_form_data(metadata, stream).unwrap();

                    // Stream data
                    if let Err(e) = client
                        .send_multipart_request(HIK8S_ROUTE_LOG, form_data)
                        .await
                    {
                        error!("Failed to send data: {}", e);
                        continue;
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(e @ RecvTimeoutError::Disconnected) => {
                return Err(e.into());
            }
        }
    }
    Ok(())
}
