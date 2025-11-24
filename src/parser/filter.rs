use std::path::Path;

/// Return true if the file has a .json extension and exists
pub fn is_json_file(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext == "json")
}
