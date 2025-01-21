use inotify::EventMask;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::time::Duration;
use tracing::error;
use tracing::info;

use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;

use super::error::EventThreadError;

use super::directory_listener::DirectoryListener;

pub fn process_file_events(
    base_path: &Path,
    sender: mpsc::Sender<HashSet<PathBuf>>,
    termination_signal: Arc<AtomicBool>,
) -> Result<(), EventThreadError> {
    info!("Starting process_file_events thread...");
    // buffer for reading close write events
    // fits 25.6 events (40 bytes per event)
    let mut buffer = [0; 65536];

    // Add a watch for each file in the directory
    let mut listener = DirectoryListener::new(sender.clone())?;
    listener.add_watches(base_path)?;

    loop {
        if termination_signal.load(Ordering::SeqCst) {
            // this sleep allows receiver to close the channel
            std::thread::sleep(Duration::from_millis(500));
            break;
        }
        let events: inotify::Events<'_> = match listener.inotify.read_events(&mut buffer) {
            Ok(events) => events,
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    // No events found
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                } else {
                    error!("{}", EventThreadError::IoError(e));
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    continue;
                }
            }
        };

        let mut files = HashSet::new();
        for event in events {
            if event.mask.contains(EventMask::Q_OVERFLOW) {
                tracing::warn!("Event queue overflowed; some events may have been lost");
            }
            if event.mask.contains(EventMask::CLOSE_WRITE) || event.mask.contains(EventMask::MODIFY)
            {
                if let Some(name) = event.name {
                    if let Some(dir_path) =
                        listener.get_descriptor(&event.wd.get_watch_descriptor_id())
                    {
                        let path = dir_path.join(name);
                        files.insert(path);
                    }
                }
            }
            if event.mask.contains(EventMask::CREATE) {
                if let Some(name) = event.name {
                    if let Some(dir_path) =
                        listener.get_descriptor(&event.wd.get_watch_descriptor_id())
                    {
                        let path = dir_path.join(name);
                        if path.is_dir() {
                            listener
                                .add_watches(&path)
                                .map_err(EventThreadError::DirectoryListener)
                                .inspect_err(|e| error!("{e}"))
                                .ok();
                        }
                    }
                }
            }
        }
        sender
            .send(files)
            .map_err(EventThreadError::SendError)
            .inspect_err(|e| error!("{e}"))
            .ok();
    }
    Ok(())
}
