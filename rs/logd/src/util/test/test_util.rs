use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub fn create_test_file(dir_path: &PathBuf, file_name: &str) -> Result<PathBuf, std::io::Error> {
    let file_path = dir_path.join(format!("{}.txt", file_name));
    let contents = format!(
        "This is the first line of {}.\nThis is the second line of {}.",
        file_name, file_name
    );
    let mut file = File::create(file_path.clone())?;
    writeln!(file, "{}", contents)?;
    Ok(file_path)
}
pub fn write_to_existing_file(file_path: &PathBuf, content: &str) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_path)?;
    writeln!(file, "{}", content)?;
    Ok(())
}
