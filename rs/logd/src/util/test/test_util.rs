use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn create_test_file<P: AsRef<Path>>(
    dir_path: P,
    file_name: &str,
) -> Result<PathBuf, std::io::Error> {
    let file_path = dir_path.as_ref().join(format!("{}.txt", file_name));
    let contents = format!(
        "This is the first line of {}.\nThis is the second line of {}.",
        file_name, file_name
    );
    let mut file = File::create(file_path.clone())?;
    writeln!(file, "{}", contents)?;
    Ok(file_path)
}
