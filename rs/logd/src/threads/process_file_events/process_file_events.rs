use inotify::{EventMask, Inotify};
use std::collections::HashSet;
use std::io::ErrorKind;

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
    let mut inotify = Inotify::init()?;

    // buffer for reading close write events
    // fits 25.6 events (40 bytes per event)
    let mut buffer = [0; 65536];

    // Add a watch for each file in the directory
    let mut listener = DirectoryListener::new(sender.clone())?;
    listener.add_watches(base_path)?;

    loop {
        if termination_signal.load(Ordering::SeqCst) {
            break;
        }
        let events: inotify::Events<'_> = match inotify.read_events(&mut buffer) {
            Ok(events) => events,
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    // No events found
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                } else {
                    Err(e)?
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
                                .expect("Failed to add directory watch");
                        }
                    }
                }
            }
        }
        sender.send(files).expect("Failed to send event");
    }
    Ok(())
}
