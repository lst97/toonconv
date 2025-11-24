use tempfile::NamedTempFile;
use std::io::Write;
use toonconv::conversion::{ConversionEngine, ConversionConfig};
use toonconv::parser::JsonSource;

#[test]
fn test_file_to_toon_success() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "{\"name\": \"Alice\", \"age\": 30}").unwrap();

    let cfg = ConversionConfig::default();
    let engine = ConversionEngine::new(cfg);

    let source = JsonSource::File(tmp.path().to_path_buf());
    let result = engine.convert_from_source(&source);
    assert!(result.is_ok());
}

#[test]
fn test_file_to_toon_invalid_json() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "{{name: 'Alice', age: 30}").unwrap(); // invalid JSON

    let cfg = ConversionConfig::default();
    let engine = ConversionEngine::new(cfg);

    let source = JsonSource::File(tmp.path().to_path_buf());
    let result = engine.convert_from_source(&source);
    assert!(result.is_err());
}
