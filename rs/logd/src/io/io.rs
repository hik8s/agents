use std::fs::File;
use std::io::{self, BufReader, Seek};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub fn get_file_position(
    path: &PathBuf,
    file_positions: &Arc<Mutex<HashMap<PathBuf, u64>>>,
) -> u64 {
    let mut file_positions_lock = file_positions.lock().expect("Failed to lock mutex");
    *file_positions_lock.entry(path.clone()).or_insert(0)
}

pub fn get_reader(mut file: File, position: u64) -> Result<BufReader<File>, io::Error> {
    file.seek(std::io::SeekFrom::Start(position))?;
    Ok(BufReader::new(file))
}

pub fn set_file_position(
    path: &PathBuf,
    file_positions: &Arc<Mutex<HashMap<PathBuf, u64>>>,
    position: u64,
) -> Result<(), io::Error> {
    let mut file_positions_lock = file_positions.lock().expect("Failed to lock mutex");
    *file_positions_lock.entry(path.clone()).or_insert(0) = position;
    Ok(())
}
