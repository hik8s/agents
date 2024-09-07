use std::io::Seek;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    sync::mpsc::Receiver,
};
use tracing::error;

use crate::client::create_form_data;
use crate::client::Hik8sClient;
use crate::{reader::read_chunk, util::io::get_reader};

use super::error::ReadThreadError;

pub async fn read_and_track_files(
    event_receiver: Receiver<HashSet<PathBuf>>,
) -> Result<(), ReadThreadError> {
    let mut positions: HashMap<PathBuf, u64> = HashMap::new();
    let client = Hik8sClient::new().unwrap();
    while let Ok(paths) = event_receiver.try_recv() {
        for path in paths {
            // Read file
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
            read_chunk(&mut reader, 1048576, data_sender)
                .map_err(|e| error!("Failed to read lines from file {}: {}", path.display(), e))
                .unwrap();

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
            client.send_multipart_request(form_data).await.unwrap();
        }
    }
    Ok(())
}
