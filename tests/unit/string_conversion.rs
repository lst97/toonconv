//! Unit tests for string-to-TOON conversion functionality
//! 
//! Tests cover:
//! - Basic JSON string to TOON conversion
//! - Edge cases (empty strings, null values, etc.)
//! - Error handling for invalid JSON
//! - Performance requirements

use toonconv::conversion::{convert_json_string, ConversionConfig};
use toonconv::error::ConversionError;
use serde_json::json;

#[cfg(test)]
mod string_conversion_tests {
    use super::*;

    /// Test basic object conversion
    #[test]
    fn test_basic_object_conversion() {
        let config = ConversionConfig::default();
        let json_str = r#"{"name": "Alice", "age": 30, "active": true}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        // Basic TOON format verification
        assert!(toon_content.contains("name:"));
        assert!(toon_content.contains("Alice"));
        assert!(toon_content.contains("age:"));
        assert!(toon_content.contains("30"));
        assert!(toon_content.contains("active:"));
        assert!(toon_content.contains("true"));
        
        // Should not contain JSON syntax
        assert!(!toon_content.contains("{"));
        assert!(!toon_content.contains("}"));
        assert!(!toon_content.contains(":"));
        assert!(!toon_content.contains("\""));
    }

    /// Test array conversion
    #[test]
    fn test_array_conversion() {
        let config = ConversionConfig::default();
        let json_str = r#"[1, 2, 3, 4, 5]"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        // Array format verification
        assert!(toon_content.starts_with("[5]:"));
        assert!(toon_content.contains("1,2,3,4,5"));
        
        // Should not contain JSON array syntax
        assert!(!toon_content.contains("["));
        assert!(!toon_content.contains("]"));
    }

    /// Test user array conversion (tabular format)
    #[test]
    fn test_user_array_conversion() {
        let config = ConversionConfig::default();
        let json_str = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        // Tabular array format verification
        assert!(toon_content.contains("[2,]{id,name}:"));
        assert!(toon_content.contains("1,Alice"));
        assert!(toon_content.contains("2,Bob"));
        
        // Should use tabular format, not JSON objects
        assert!(!toon_content.contains("{\"id\":"));
        assert!(!toon_content.contains("\"name\":"));
    }

    /// Test nested object conversion
    #[test]
    fn test_nested_object_conversion() {
        let config = ConversionConfig::default();
        let json_str = r#"{"user": {"name": "Alice", "settings": {"theme": "dark"}}}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        // Nested structure verification
        assert!(toon_content.contains("user:"));
        assert!(toon_content.contains("name:"));
        assert!(toon_content.contains("Alice"));
        assert!(toon_content.contains("settings:"));
        assert!(toon_content.contains("theme:"));
        assert!(toon_content.contains("dark"));
        
        // Should have proper indentation for nested structure
        let lines: Vec<&str> = toon_content.lines().collect();
        assert!(lines.len() >= 3); // Should have multiple lines due to nesting
    }

    /// Test empty object
    #[test]
    fn test_empty_object() {
        let config = ConversionConfig::default();
        let json_str = r#"{}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        assert_eq!(toon_content.trim(), "{}");
    }

    /// Test empty array
    #[test]
    fn test_empty_array() {
        let config = ConversionConfig::default();
        let json_str = r#"[]"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        assert_eq!(toon_content.trim(), "[]");
    }

    /// Test null values
    #[test]
    fn test_null_values() {
        let config = ConversionConfig::default();
        let json_str = r#"{"value": null}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        assert!(toon_content.contains("value:"));
        assert!(toon_content.contains("null"));
    }

    /// Test number types
    #[test]
    fn test_number_types() {
        let config = ConversionConfig::default();
        let json_str = r#"{"integer": 42, "float": 3.14, "negative": -10}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        assert!(toon_content.contains("integer:"));
        assert!(toon_content.contains("42"));
        assert!(toon_content.contains("float:"));
        assert!(toon_content.contains("3.14"));
        assert!(toon_content.contains("negative:"));
        assert!(toon_content.contains("-10"));
    }

    /// Test boolean values
    #[test]
    fn test_boolean_values() {
        let config = ConversionConfig::default();
        let json_str = r#"{"active": true, "inactive": false}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        assert!(toon_content.contains("active:"));
        assert!(toon_content.contains("true"));
        assert!(toon_content.contains("inactive:"));
        assert!(toon_content.contains("false"));
    }

    /// Test invalid JSON syntax
    #[test]
    fn test_invalid_json_syntax() {
        let config = ConversionConfig::default();
        let invalid_json = r#"{"name": "test", "value": }"#;
        
        let result = convert_json_string(invalid_json, &config);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("JSON parse error"));
    }

    /// Test unterminated JSON string
    #[test]
    fn test_unterminated_json_string() {
        let config = ConversionConfig::default();
        let invalid_json = r#"{"name": "test"#;
        
        let result = convert_json_string(invalid_json, &config);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("JSON parse error"));
    }

    /// Test completely empty string
    #[test]
    fn test_empty_input_string() {
        let config = ConversionConfig::default();
        let empty_json = "";
        
        let result = convert_json_string(empty_json, &config);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Empty JSON string"));
    }

    /// Test whitespace-only string
    #[test]
    fn test_whitespace_only_string() {
        let config = ConversionConfig::default();
        let whitespace_json = "   \n\t  ";
        
        let result = convert_json_string(whitespace_json, &config);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Empty JSON string"));
    }

    /// Test string with smart quoting rules
    #[test]
    fn test_string_quoting_rules() {
        let config = ConversionConfig::default();
        let json_str = r#"{"empty": "", "keyword": "true", "number": "42", "normal": "hello"}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        // Empty string should be quoted
        assert!(toon_content.contains("empty: \"\""));
        
        // Keyword should be quoted
        assert!(toon_content.contains("keyword: \"true\""));
        
        // Numeric string should be quoted
        assert!(toon_content.contains("number: \"42\""));
        
        // Normal string should not be quoted
        assert!(toon_content.contains("normal: hello"));
    }

    /// Test performance requirement (< 1 second for strings < 1MB)
    #[test]
    fn test_performance_requirement() {
        use std::time::Instant;
        
        let config = ConversionConfig::default();
        
        // Create a JSON string approaching 1MB
        let mut json_parts = Vec::new();
        for i in 0..10000 {
            json_parts.push(format!(r#"{{"id": {}, "name": "user{}", "active": true}}"#, i, i));
        }
        let large_json = format!("[{}]", json_parts.join(","));
        
        // Ensure it's under 1MB
        assert!(large_json.len() < 1_000_000);
        
        let start_time = Instant::now();
        let result = convert_json_string(&large_json, &config);
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok(), "Conversion should succeed");
        assert!(elapsed.as_secs_f32() < 1.0, 
               "Conversion should complete in under 1 second, took {:?}", elapsed);
        
        // Verify the output is valid TOON
        let toon_data = result.unwrap();
        assert!(!toon_data.content.is_empty());
    }

    /// Test conversion metadata
    #[test]
    fn test_conversion_metadata() {
        let config = ConversionConfig::default();
        let json_str = r#"{"name": "Alice", "age": 30}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        
        // Check metadata
        assert!(toon_data.metadata.input_size > 0);
        assert!(toon_data.metadata.output_size > 0);
        assert!(toon_data.metadata.processing_time_ms > 0);
        assert!(toon_data.metadata.token_reduction >= 0.0);
        
        // For simple objects, should show some token reduction
        assert!(toon_data.metadata.token_reduction > 0.0);
    }

    /// Test with custom configuration
    #[test]
    fn test_custom_configuration() {
        let mut config = ConversionConfig::default();
        config.indent_size = 4;
        config.delimiter = crate::conversion::DelimiterType::Tab;
        config.length_marker = true;
        
        let json_str = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        // Should use tab delimiter
        assert!(toon_content.contains('\t'));
        
        // Should contain length markers
        assert!(toon_content.contains("[2"));
    }

    /// Test complex mixed data types
    #[test]
    fn test_complex_mixed_data() {
        let config = ConversionConfig::default();
        let json_str = r#"{
  "metadata": {
    "version": 1,
    "author": "system"
  },
  "users": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"}
  ],
  "tags": ["urgent", "pending"],
  "config": {
    "timeout": 30,
    "retries": 3
  }
}"#;
        
        let result = convert_json_string(json_str, &config);
        assert!(result.is_ok());
        
        let toon_data = result.unwrap();
        let toon_content = toon_data.content;
        
        // Should handle nested objects
        assert!(toon_content.contains("metadata:"));
        assert!(toon_content.contains("version:"));
        assert!(toon_content.contains("1"));
        
        // Should use tabular format for uniform object array
        assert!(toon_content.contains("[2,]{id,name}:"));
        assert!(toon_content.contains("1,Alice"));
        assert!(toon_content.contains("2,Bob"));
        
        // Should handle primitive arrays
        assert!(toon_content.contains("tags"));
        assert!(toon_content.contains("urgent,pending"));
        
        // Should handle nested config object
        assert!(toon_content.contains("config:"));
        assert!(toon_content.contains("timeout:"));
        assert!(toon_content.contains("30"));
    }
}