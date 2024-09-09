use inotify::{Inotify, WatchMask};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use tracing::info;

use super::error::DirectoryListenerError;

pub struct DirectoryListener {
    inotify: Inotify,
    pub watch_descriptors: HashMap<i32, PathBuf>,
    sender: Sender<HashSet<PathBuf>>,
}

impl DirectoryListener {
    pub fn new(sender: Sender<HashSet<PathBuf>>) -> Result<Self, DirectoryListenerError> {
        let inotify = Inotify::init()?;
        Ok(Self {
            inotify,
            watch_descriptors: HashMap::new(),
            sender,
        })
    }

    pub fn get_descriptor(&self, watch_descriptor_id: &i32) -> Option<&PathBuf> {
        self.watch_descriptors.get(watch_descriptor_id)
    }

    pub fn add_watches(&mut self, path: &Path) -> Result<(), DirectoryListenerError> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                self.add_watches(&path)?;
            }
        }

        info!("Adding watch for {:?}", path);
        let watch = self.inotify.watches().add(
            path,
            WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::CLOSE_WRITE,
        )?;

        self.watch_descriptors
            .insert(watch.get_watch_descriptor_id(), path.to_path_buf());

        // Send the path of the file that was added
        if path.is_file() {
            let mut paths = HashSet::new();
            paths.insert(path.to_path_buf());
            self.sender.send(paths)?;
        }

        Ok(())
    }
}
