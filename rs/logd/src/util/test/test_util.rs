use std::fs::File;
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
