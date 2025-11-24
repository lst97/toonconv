use std::path::Path;

/// Return true if the file has a .json extension and exists
pub fn is_json_file(path: &Path) -> bool {
    path.is_file() && path.extension().map_or(false, |ext| ext == "json")
}
