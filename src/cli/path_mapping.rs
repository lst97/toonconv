use std::path::{PathBuf};

/// Map an input JSON file into an output TOON file path.
/// This preserves the input directory structure relative to `input_dir`.
pub fn map_input_to_output(input_dir: &PathBuf, input_file: &PathBuf, output_dir: &PathBuf, extension: &str) -> PathBuf {
    let relative = input_file.strip_prefix(input_dir).unwrap_or(input_file);
    let mut out = output_dir.join(relative);
    out.set_extension(extension);
    out
}
