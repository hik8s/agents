use inotify::{Inotify, WatchMask};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, Seek};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::{collections::HashMap, path::PathBuf};

pub fn get_reader(mut file: File, position: u64) -> Result<BufReader<File>, io::Error> {
    file.seek(std::io::SeekFrom::Start(position))?;
    Ok(BufReader::new(file))
}

pub fn add_watches(
    inotify: &mut Inotify,
    path: &Path,
    map: &mut HashMap<i32, PathBuf>,
    tx: &Sender<HashSet<PathBuf>>,
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
