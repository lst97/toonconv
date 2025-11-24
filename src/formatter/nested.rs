//! Nested object formatting with proper indentation hierarchy
//!
//! This module handles deeply nested JSON structures and maintains
//! proper indentation levels for TOON output.

use crate::conversion::ConversionConfig;
use crate::error::{FormattingError, FormattingResult};
use serde_json::{Map, Value};

/// Maximum nesting depth to prevent stack overflow
const MAX_NESTING_DEPTH: usize = 100;

/// Nested object formatter with indentation tracking
pub struct NestedFormatter<'a> {
    config: &'a ConversionConfig,
    current_depth: usize,
}

impl<'a> NestedFormatter<'a> {
    /// Create a new nested formatter
    pub fn new(config: &'a ConversionConfig) -> Self {
        Self {
            config,
            current_depth: 0,
        }
    }

    /// Format a nested object with proper indentation
    pub fn format_nested_object(
        &mut self,
        object: &Map<String, Value>,
    ) -> FormattingResult<String> {
        self.check_depth()?;

        if object.is_empty() {
            return Ok("{}".to_string());
        }

        let mut result = String::new();
        result.push('{');

        let keys: Vec<&str> = object.keys().map(|k| k.as_str()).collect();

        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                result.push(',');
            }

            result.push('\n');
            result.push_str(&self.get_indent());

            // Format key with quoting if needed
            let formatted_key = self.format_key(key)?;
            result.push_str(&formatted_key);
            result.push_str(": ");

            // Format value with increased nesting
            let value = object.get(*key).unwrap();
            self.current_depth += 1;
            let formatted_value = self.format_nested_value(value)?;
            self.current_depth -= 1;

            result.push_str(&formatted_value);
        }

        result.push('\n');
        result.push_str(&self.get_indent());
        result.push('}');

        Ok(result)
    }

    /// Format a nested value (recursive)
    pub fn format_nested_value(&mut self, value: &Value) -> FormattingResult<String> {
        self.check_depth()?;

        match value {
            Value::Null => Ok("null".to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Number(n) => self.format_number(n),
            Value::String(s) => self.format_string(s),
            Value::Array(a) => self.format_nested_array(a),
            Value::Object(o) => self.format_nested_object(o),
        }
    }

    /// Format a nested array
    fn format_nested_array(&mut self, array: &[Value]) -> FormattingResult<String> {
        self.check_depth()?;

        if array.is_empty() {
            return Ok("[]".to_string());
        }

        // For nested contexts, use compact JSON-style formatting
        let mut result = String::new();
        result.push('[');

        for (i, value) in array.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }

            self.current_depth += 1;
            let formatted = self.format_nested_value(value)?;
            self.current_depth -= 1;

            result.push_str(&formatted);
        }

        result.push(']');
        Ok(result)
    }

    /// Format a number value
    fn format_number(&self, value: &serde_json::Number) -> FormattingResult<String> {
        if let Some(f) = value.as_f64() {
            if f.is_infinite() {
                return Err(FormattingError::invalid_structure(
                    "Infinite numbers are not supported in TOON".to_string(),
                ));
            }
            if f.is_nan() {
                return Err(FormattingError::invalid_structure(
                    "NaN values are not supported in TOON".to_string(),
                ));
            }
        }
        Ok(value.to_string())
    }

    /// Format a string value with smart quoting
    fn format_string(&self, value: &str) -> FormattingResult<String> {
        use crate::conversion::QuoteStrategy;

        match self.config.quote_strings {
            QuoteStrategy::Always => self.quote_string(value),
            QuoteStrategy::Never => Ok(value.to_string()),
            QuoteStrategy::Smart => {
                if self.should_quote_string(value) {
                    self.quote_string(value)
                } else {
                    Ok(value.to_string())
                }
            }
        }
    }

    /// Format an object key
    fn format_key(&self, key: &str) -> FormattingResult<String> {
        if self.should_quote_string(key) {
            self.quote_string(key)
        } else {
            Ok(key.to_string())
        }
    }

    /// Check if a string needs quoting
    fn should_quote_string(&self, value: &str) -> bool {
        if value.is_empty() {
            return true;
        }

        // Keywords that need quoting
        if value == "null" || value == "true" || value == "false" {
            return true;
        }

        // Numeric strings
        if value.parse::<f64>().is_ok() {
            return true;
        }

        // Leading/trailing whitespace
        if value.trim() != value {
            return true;
        }

        // TOON control characters: colon, comma, newline, braces, brackets
        let control_chars = ":[]{}\n\r";
        if value.chars().any(|c| control_chars.contains(c)) {
            return true;
        }

        // Contains delimiter
        if value.contains(self.config.delimiter.as_str()) {
            return true;
        }

        // Control characters
        if value.chars().any(|c| c.is_control()) {
            return true;
        }

        false
    }

    /// Quote a string according to TOON rules
    fn quote_string(&self, value: &str) -> FormattingResult<String> {
        if value.is_empty() {
            return Ok("\"\"".to_string());
        }

        let mut quoted = String::with_capacity(value.len() + 2);
        quoted.push('"');

        for ch in value.chars() {
            match ch {
                '"' => quoted.push_str("\\\""),
                '\\' => quoted.push_str("\\\\"),
                '\n' => quoted.push_str("\\n"),
                '\r' => quoted.push_str("\\r"),
                '\t' => quoted.push_str("\\t"),
                _ => quoted.push(ch),
            }
        }

        quoted.push('"');
        Ok(quoted)
    }

    /// Get current indentation string
    fn get_indent(&self) -> String {
        " ".repeat(self.current_depth * self.config.indent_size as usize)
    }

    /// Check if we've exceeded maximum nesting depth
    fn check_depth(&self) -> FormattingResult<()> {
        if self.current_depth >= MAX_NESTING_DEPTH {
            return Err(FormattingError::invalid_structure(format!(
                "Maximum nesting depth exceeded ({})",
                MAX_NESTING_DEPTH
            )));
        }
        Ok(())
    }

    /// Get current nesting depth (for testing/debugging)
    pub fn current_depth(&self) -> usize {
        self.current_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_nested_object() {
        let config = ConversionConfig::default();
        let mut formatter = NestedFormatter::new(&config);

        let obj = json!({
            "user": {
                "name": "Alice",
                "age": 30
            }
        });

        let result = formatter
            .format_nested_object(obj.as_object().unwrap())
            .unwrap();
        assert!(result.contains("user:"));
        assert!(result.contains("name:"));
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_deeply_nested_object() {
        let config = ConversionConfig::default();
        let mut formatter = NestedFormatter::new(&config);

        let obj = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": "deep"
                    }
                }
            }
        });

        let result = formatter
            .format_nested_object(obj.as_object().unwrap())
            .unwrap();
        assert!(result.contains("level1:"));
        assert!(result.contains("level2:"));
        assert!(result.contains("level3:"));
        assert!(result.contains("value:"));
        assert!(result.contains("deep"));
    }

    #[test]
    fn test_max_depth_exceeded() {
        let config = ConversionConfig::default();
        let mut formatter = NestedFormatter::new(&config);

        // Artificially set depth to maximum
        formatter.current_depth = MAX_NESTING_DEPTH;

        let obj = json!({"key": "value"});
        let result = formatter.format_nested_object(obj.as_object().unwrap());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Maximum nesting depth"));
    }

    #[test]
    fn test_nested_arrays_in_objects() {
        let config = ConversionConfig::default();
        let mut formatter = NestedFormatter::new(&config);

        let obj = json!({
            "data": {
                "items": [1, 2, 3],
                "nested": {
                    "values": [4, 5, 6]
                }
            }
        });

        let result = formatter
            .format_nested_object(obj.as_object().unwrap())
            .unwrap();
        assert!(result.contains("data:"));
        assert!(result.contains("items:"));
        assert!(result.contains("[1, 2, 3]"));
        assert!(result.contains("nested:"));
        assert!(result.contains("values:"));
    }

    #[test]
    fn test_empty_nested_objects() {
        let config = ConversionConfig::default();
        let mut formatter = NestedFormatter::new(&config);

        let obj = json!({
            "empty": {},
            "other": "value"
        });

        let result = formatter
            .format_nested_object(obj.as_object().unwrap())
            .unwrap();
        assert!(result.contains("empty: {}"));
        assert!(result.contains("other:"));
    }

    #[test]
    fn test_string_quoting_in_nested() {
        let config = ConversionConfig::default();
        let mut formatter = NestedFormatter::new(&config);

        let obj = json!({
            "data": {
                "keyword": "true",
                "normal": "hello",
                "with:colon": "value"
            }
        });

        let result = formatter
            .format_nested_object(obj.as_object().unwrap())
            .unwrap();
        assert!(result.contains("\"true\"")); // keyword quoted
        assert!(result.contains("hello")); // normal not quoted
        assert!(result.contains("\"with:colon\"")); // key with colon quoted
    }
}
