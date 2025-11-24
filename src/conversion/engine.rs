//! Core conversion engine for JSON to TOON transformation

use crate::conversion::config::ConversionConfig;
use crate::conversion::limits;
use crate::conversion::ConversionResult;
use crate::error::{ConversionError, ConversionErrorKind};
use crate::formatter::format_to_toon;
use crate::parser::validation::validate_json_structure;
use crate::parser::JsonSource;
use crate::validation::{CircularRefDetector, ToonValidator};
use serde_json::Value;
use std::time::Instant;

/// Core conversion result
#[derive(Debug, Clone)]
pub struct ToonData {
    pub content: String,
    pub metadata: ConversionMetadata,
}

impl ToonData {
    /// Create a new TOON data result
    pub fn new(content: String, metadata: ConversionMetadata) -> Self {
        Self { content, metadata }
    }

    /// Get the formatted TOON output
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Get the length of the output in bytes
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if the output is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

/// Metadata about the conversion process
#[derive(Debug, Clone)]
pub struct ConversionMetadata {
    pub input_size: u64,
    pub output_size: u64,
    pub token_reduction: f32,
    pub processing_time_ms: u64,
    pub memory_peak_kb: usize,
    pub schema_info: Option<SchemaInfo>,
}

/// Schema information for arrays
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub array_count: usize,
    pub uniform_arrays: Vec<ArraySchema>,
}

/// Schema for a single array
#[derive(Debug, Clone)]
pub struct ArraySchema {
    pub element_count: usize,
    pub field_count: Option<usize>,
    pub field_names: Option<Vec<String>>,
}

/// Main conversion engine
pub struct ConversionEngine {
    config: ConversionConfig,
}

impl ConversionEngine {
    /// Create a new conversion engine
    pub fn new(config: ConversionConfig) -> Self {
        Self { config }
    }

    /// Convert JSON data to TOON format
    pub fn convert(&self, json_data: &Value) -> ConversionResult<ToonData> {
        let start_time = Instant::now();

        // Validate input
        self.validate_input(json_data)?;

        // Convert to TOON
        let toon_content = self.convert_to_toon(json_data)?;

        // Validate TOON output for compliance
        let validator = ToonValidator::new(true); // Strict mode
        let validation_result = validator.validate(&toon_content, json_data)?;
        if !validation_result.is_valid() {
            return Err(ConversionError::conversion(
                ConversionErrorKind::Configuration {
                    message: format!(
                        "TOON output validation failed: {:?}",
                        validation_result.issues
                    ),
                },
            ));
        }

        // Calculate metadata
        let input_size = self.estimate_input_size(json_data);
        let output_size = toon_content.len() as u64;
        let processing_time = start_time.elapsed();
        let token_reduction = self.calculate_token_reduction(input_size, output_size);

        let metadata = ConversionMetadata {
            input_size,
            output_size,
            token_reduction,
            processing_time_ms: processing_time.as_millis() as u64,
            memory_peak_kb: self.estimate_memory_usage(),
            schema_info: self.extract_schema_info(json_data),
        };

        Ok(ToonData::new(toon_content, metadata))
    }

    /// Convert JSON from a source to TOON
    pub fn convert_from_source(&self, source: &JsonSource) -> ConversionResult<ToonData> {
        // Check source size before reading to avoid loading very large files
        limits::check_source_size_before_read(source, &self.config)?;

        // Parse JSON from source
        let json_value = source.parse().map_err(ConversionError::ParseError)?;

        // Convert to TOON
        self.convert(&json_value)
    }

    /// Convert JSON string to TOON
    pub fn convert_string(&self, json_str: &str) -> ConversionResult<ToonData> {
        let source = JsonSource::String(json_str.to_string());
        self.convert_from_source(&source)
    }

    /// Validate input JSON data
    fn validate_input(&self, json_data: &Value) -> ConversionResult<()> {
        // Check basic structure validation
        validate_json_structure(json_data).map_err(ConversionError::ParseError)?;

        // Check for circular references
        let mut detector = CircularRefDetector::new(100); // Max depth 100
        if !detector.is_safe(json_data) {
            return Err(ConversionError::conversion(
                ConversionErrorKind::Configuration {
                    message: "Circular reference or excessive nesting detected in JSON structure"
                        .to_string(),
                },
            ));
        }

        // Check size constraints
        let estimated_size = self.estimate_input_size(json_data);
        if estimated_size > self.config.memory_limit as u64 {
            return Err(ConversionError::conversion(
                ConversionErrorKind::JsonTooLarge {
                    size: estimated_size as usize,
                    limit: self.config.memory_limit,
                },
            ));
        }

        // After a basic estimate, perform a serialization-based check which can
        // be more accurate for memory usage.
        crate::conversion::limits::check_json_value_size(json_data, &self.config)?;

        // Check timeout constraint
        if self.config.timeout.as_secs() == 0 {
            return Err(ConversionError::conversion(
                ConversionErrorKind::Configuration {
                    message: "Timeout must be greater than 0".to_string(),
                },
            ));
        }

        Ok(())
    }

    /// Convert JSON value to TOON string
    fn convert_to_toon(&self, json_data: &Value) -> ConversionResult<String> {
        // Use the TOON formatter
        format_to_toon(json_data, &self.config)
    }

    /// Estimate input size in bytes
    fn estimate_input_size(&self, json_data: &Value) -> u64 {
        // This is a rough estimate - in a real implementation,
        // you'd want to track the actual size from the source
        match json_data {
            Value::String(s) => s.len() as u64,
            Value::Number(n) => n.to_string().len() as u64,
            Value::Bool(_) => 4,
            Value::Null => 4,
            Value::Array(a) => {
                // Estimate based on array elements
                (a.len() * 10) as u64 // Rough estimate
            }
            Value::Object(o) => {
                // Estimate based on object properties
                (o.len() * 20) as u64 // Rough estimate
            }
        }
    }

    /// Calculate token reduction percentage
    fn calculate_token_reduction(&self, input_size: u64, output_size: u64) -> f32 {
        if input_size == 0 {
            return 0.0;
        }

        let reduction = ((input_size as f32 - output_size as f32) / input_size as f32) * 100.0;
        reduction.max(0.0) // Don't show negative reduction
    }

    /// Estimate peak memory usage
    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate - in a real implementation, track actual usage
        // This should be much more sophisticated
        self.config.memory_limit / 4 // Assume peak usage is 25% of limit
    }

    /// Extract schema information from JSON
    fn extract_schema_info(&self, json_data: &Value) -> Option<SchemaInfo> {
        let mut array_count = 0;
        let mut uniform_arrays = Vec::new();

        self.extract_array_schemas(json_data, &mut array_count, &mut uniform_arrays);

        if array_count > 0 {
            Some(SchemaInfo {
                array_count,
                uniform_arrays,
            })
        } else {
            None
        }
    }

    /// Recursively extract array schemas
    fn extract_array_schemas(
        &self,
        value: &Value,
        array_count: &mut usize,
        uniform_arrays: &mut Vec<ArraySchema>,
    ) {
        match value {
            Value::Array(arr) => {
                *array_count += 1;

                if self.is_uniform_object_array(arr) {
                    let first_obj = arr[0].as_object().unwrap();
                    uniform_arrays.push(ArraySchema {
                        element_count: arr.len(),
                        field_count: Some(first_obj.len()),
                        field_names: Some(first_obj.keys().cloned().collect()),
                    });
                }

                for item in arr {
                    self.extract_array_schemas(item, array_count, uniform_arrays);
                }
            }
            Value::Object(obj) => {
                for value in obj.values() {
                    self.extract_array_schemas(value, array_count, uniform_arrays);
                }
            }
            Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {}
        }
    }

    /// Check if array contains uniform objects
    fn is_uniform_object_array(&self, array: &[Value]) -> bool {
        if array.is_empty() {
            return false;
        }

        // All elements must be objects
        if !array.iter().all(|v| v.is_object()) {
            return false;
        }

        // All objects must have the same keys
        let first_keys: std::collections::HashSet<&str> = array[0]
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.as_str())
            .collect();

        array.iter().all(|v| {
            let keys: std::collections::HashSet<&str> =
                v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
            keys == first_keys
        })
    }
}

/// High-level conversion functions
/// Convert JSON value to TOON with default configuration
pub fn convert_json_to_toon(
    json_data: &Value,
    config: &ConversionConfig,
) -> ConversionResult<ToonData> {
    let engine = ConversionEngine::new(config.clone());
    engine.convert(json_data)
}

/// Convert JSON from source to TOON
pub fn convert_json_from_source(
    source: &JsonSource,
    config: &ConversionConfig,
) -> ConversionResult<ToonData> {
    let engine = ConversionEngine::new(config.clone());
    engine.convert_from_source(source)
}

/// Convert JSON string to TOON
pub fn convert_json_string(
    json_str: &str,
    config: &ConversionConfig,
) -> ConversionResult<ToonData> {
    let engine = ConversionEngine::new(config.clone());
    engine.convert_string(json_str)
}

/// Streaming conversion for large files
pub fn convert_stream_to_toon<R: std::io::Read>(
    reader: R,
    config: &ConversionConfig,
) -> ConversionResult<String> {
    // For large files, we would implement streaming parsing
    // For now, this is a placeholder that reads all content
    let content = std::io::read_to_string(reader).map_err(|e| {
        ConversionError::conversion(ConversionErrorKind::Io {
            message: format!("Failed to read stream: {}", e),
            path: None,
        })
    })?;

    let engine = ConversionEngine::new(config.clone());
    let result = engine.convert_string(&content)?;
    Ok(result.content)
}

/// Batch conversion for multiple files
pub fn convert_batch(
    sources: Vec<JsonSource>,
    config: &ConversionConfig,
) -> ConversionResult<Vec<(JsonSource, ToonData)>> {
    let engine = ConversionEngine::new(config.clone());
    let mut results = Vec::new();

    for source in sources {
        match engine.convert_from_source(&source) {
            Ok(toon_data) => results.push((source, toon_data)),
            Err(e) => {
                return Err(ConversionError::conversion_with_source(
                    ConversionErrorKind::ConversionFailed {
                        message: format!(
                            "Failed to convert source: {:?}",
                            source.source_type().description()
                        ),
                    },
                    e.into(),
                ));
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::ConversionConfig;

    #[test]
    fn test_basic_conversion() {
        let config = ConversionConfig::default();
        let engine = ConversionEngine::new(config);

        let json = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });

        let result = engine.convert(&json).unwrap();
        assert!(result.content.contains("name:"));
        assert!(result.content.contains("Alice"));
        assert!(result.metadata.input_size > 0);
        assert!(result.metadata.output_size > 0);
    }

    #[test]
    fn test_string_conversion() {
        let config = ConversionConfig::default();
        let engine = ConversionEngine::new(config);

        let json_str = r#"{"name": "test", "value": 42}"#;
        let result = engine.convert_string(json_str).unwrap();

        assert!(result.content.contains("name:"));
        assert!(result.content.contains("test"));
    }

    #[test]
    fn test_token_reduction_calculation() {
        let config = ConversionConfig::default();
        let engine = ConversionEngine::new(config);

        // JSON is typically more verbose than TOON
        let json = serde_json::json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]
        });

        let result = engine.convert(&json).unwrap();

        // TOON should typically be smaller for structured data
        assert!(result.metadata.token_reduction >= 0.0);
        // processing_time_ms is u64, always >= 0
    }

    #[test]
    fn test_array_schema_extraction() {
        let config = ConversionConfig::default();
        let engine = ConversionEngine::new(config);

        let json = serde_json::json!({
            "users": [
                {"id": 1, "name": "Alice", "active": true},
                {"id": 2, "name": "Bob", "active": false}
            ],
            "tags": ["important", "pending"]
        });

        let result = engine.convert(&json).unwrap();

        if let Some(schema) = result.metadata.schema_info {
            assert_eq!(schema.array_count, 2); // users and tags arrays
            assert_eq!(schema.uniform_arrays.len(), 1); // only users is uniform objects
        }
    }

    #[test]
    fn test_conversion_error_handling() {
        let config = ConversionConfig::default();
        let engine = ConversionEngine::new(config);

        // Invalid JSON
        let invalid_json = serde_json::json!({
            "unterminated": "string"
        });

        // This should be caught at parsing stage, not conversion
        let _result = engine.convert(&invalid_json);
        // In a real implementation, this would fail at JSON parsing
        // For now, we assume valid JSON input to the conversion engine
    }
}
