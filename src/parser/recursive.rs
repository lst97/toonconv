use std::path::PathBuf;
use walkdir::WalkDir;

/// Find JSON files recursively under the specified directory
pub fn find_json_files_recursive(dir: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut json_files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        if crate::parser::filter::is_json_file(path) {
            json_files.push(path.to_path_buf());
        }
    }

    Ok(json_files)
}
