use inotify::{EventMask, Inotify};
use std::collections::HashMap;
use std::collections::HashSet;

use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::constant::LOG_PATH;
use crate::io::add_watches;

use super::error::EventThreadError;

pub fn process_inotify_events(
    sender: mpsc::Sender<HashSet<PathBuf>>,
) -> Result<(), EventThreadError> {
    let mut inotify = Inotify::init()?;

    // buffer for reading close write events
    // fits 25.6 events (40 bytes per event)
    let mut buffer = [0; 65536];

    // Add a watch for each file in the directory
    let mut watch_descriptors = HashMap::new();
    add_watches(
        &mut inotify,
        Path::new(LOG_PATH),
        &mut watch_descriptors,
        &sender,
    )?;

    loop {
        let events: inotify::Events<'_> = inotify
            .read_events_blocking(&mut buffer)
            .expect("Error while reading events");

        let mut paths = HashSet::new();
        for event in events {
            if event.mask.contains(EventMask::Q_OVERFLOW) {
                tracing::warn!("Event queue overflowed; some events may have been lost");
            }
            if event.mask.contains(EventMask::CLOSE_WRITE) || event.mask.contains(EventMask::MODIFY)
            {
                if let Some(name) = event.name {
                    if let Some(dir_path) =
                        watch_descriptors.get(&event.wd.get_watch_descriptor_id())
                    {
                        let path = dir_path.join(name);
                        paths.insert(path);
                    }
                }
            }
            if event.mask.contains(EventMask::CREATE) {
                if let Some(name) = event.name {
                    if let Some(dir_path) =
                        watch_descriptors.get(&event.wd.get_watch_descriptor_id())
                    {
                        let path = dir_path.join(name);
                        if path.is_dir() {
                            add_watches(&mut inotify, &path, &mut watch_descriptors, &sender)
                                .expect("Failed to add directory watch");
                        }
                    }
                }
            }
        }
        // Convert the HashSet into a Vec and send it over the channel
        sender.send(paths).expect("Failed to send event");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
