use std::path::PathBuf;
// use crate::error::ParseResult;
use std::fs;
use walkdir::WalkDir;

/// Find JSON files in a directory. If recursive is true, use walkdir; otherwise list files.
pub fn find_json_files(dir: &PathBuf, recursive: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut json_files = Vec::new();

    if recursive {
        for entry in WalkDir::new(dir) {
            let entry = entry?;
            let path = entry.path();
            if crate::parser::filter::is_json_file(path) {
                json_files.push(path.to_path_buf());
            }
        }
    } else {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if crate::parser::filter::is_json_file(&path) {
                json_files.push(path);
            }
        }
    }

    Ok(json_files)
}
