use crate::conversion::config::ConversionConfig;
use crate::conversion::ConversionResult;
use crate::error::{ConversionError, ConversionErrorKind};
use crate::parser::JsonSource;
use serde_json::Value;

/// Check the source size before attempting to read or parse the JSON.
/// This avoids loading very large files into memory if the user-configured
/// limit is smaller than the file.
pub fn check_source_size_before_read(
    source: &JsonSource,
    config: &ConversionConfig,
) -> ConversionResult<()> {
    let source_type = source.source_type();

    if let Some(size) = source_type.estimated_size() {
        if size > config.memory_limit as u64 {
            return Err(ConversionError::conversion(
                ConversionErrorKind::JsonTooLarge {
                    size: size as usize,
                    limit: config.memory_limit,
                },
            ));
        }
    }

    Ok(())
}

/// After parsing a JSON value, check the estimated memory usage and file size
/// to ensure conversion will respect the configured limits.
pub fn check_json_value_size(json: &Value, config: &ConversionConfig) -> ConversionResult<()> {
    // Try to get the serialized length as an approximate measure
    match serde_json::to_string(json) {
        Ok(s) => {
            let len = s.len() as u64;
            if len > config.memory_limit as u64 {
                return Err(ConversionError::conversion(
                    ConversionErrorKind::MemoryLimitExceeded {
                        size: len as usize,
                        limit: config.memory_limit,
                    },
                ));
            }
        }
        Err(_) => {
            // If serialization fails, conservatively allow conversion to proceed
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_check_source_size_before_read_small() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "{{\"a\": 1}}").unwrap();

        let source = JsonSource::File(tmp.path().to_path_buf());
        let cfg = ConversionConfig {
            memory_limit: 1024 * 1024, // 1MB
            ..Default::default()
        };

        assert!(check_source_size_before_read(&source, &cfg).is_ok());
    }

    #[test]
    fn test_check_source_size_before_read_large() {
        let mut tmp = NamedTempFile::new().unwrap();
        // Write a file slightly larger than limit
        let payload = vec![b'a'; 1024 * 1024 + 10];
        tmp.write_all(&payload).unwrap();

        let source = JsonSource::File(tmp.path().to_path_buf());
        let cfg = ConversionConfig {
            memory_limit: 1024 * 1024, // 1MB
            ..Default::default()
        };

        let res = check_source_size_before_read(&source, &cfg);
        assert!(matches!(
            res.unwrap_err(),
            ConversionError::Conversion { .. }
        ));
    }

    #[test]
    fn test_check_json_value_size_exceeds() {
        let cfg = ConversionConfig {
            memory_limit: 10, // very small
            ..Default::default()
        };

        let big_value = Value::String("a".repeat(100));

        let res = check_json_value_size(&big_value, &cfg);
        assert!(matches!(
            res.unwrap_err(),
            ConversionError::Conversion { .. }
        ));
    }
}
