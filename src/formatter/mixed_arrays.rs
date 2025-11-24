//! Mixed-type array formatting for TOON
//!
//! Handles arrays with heterogeneous element types, providing
//! appropriate formatting based on content variety.

use crate::conversion::ConversionConfig;
use crate::error::{FormattingError, FormattingResult};
use serde_json::Value;

/// Mixed array formatter for heterogeneous arrays
pub struct MixedArrayFormatter<'a> {
    config: &'a ConversionConfig,
    indent_level: usize,
}

impl<'a> MixedArrayFormatter<'a> {
    /// Create a new mixed array formatter
    pub fn new(config: &'a ConversionConfig) -> Self {
        Self {
            config,
            indent_level: 0,
        }
    }

    /// Set the current indentation level
    pub fn set_indent_level(&mut self, level: usize) {
        self.indent_level = level;
    }

    /// Format a mixed-type array
    pub fn format_mixed_array(&mut self, array: &[Value]) -> FormattingResult<String> {
        if array.is_empty() {
            return Ok("[]".to_string());
        }

        // Analyze array content
        let analysis = self.analyze_array(array);

        // Choose formatting strategy based on content
        match analysis.array_type {
            ArrayType::AllPrimitives => self.format_primitive_array(array),
            ArrayType::AllObjects => self.format_object_array(array),
            ArrayType::AllArrays => self.format_nested_arrays(array),
            ArrayType::Mixed => self.format_truly_mixed_array(array),
        }
    }

    /// Analyze array content to determine type distribution
    fn analyze_array(&self, array: &[Value]) -> ArrayAnalysis {
        let mut null_count = 0;
        let mut bool_count = 0;
        let mut number_count = 0;
        let mut string_count = 0;
        let mut array_count = 0;
        let mut object_count = 0;

        for value in array {
            match value {
                Value::Null => null_count += 1,
                Value::Bool(_) => bool_count += 1,
                Value::Number(_) => number_count += 1,
                Value::String(_) => string_count += 1,
                Value::Array(_) => array_count += 1,
                Value::Object(_) => object_count += 1,
            }
        }

        let primitive_count = null_count + bool_count + number_count + string_count;
        let total = array.len();

        // Determine array type
        let array_type = if object_count == total {
            ArrayType::AllObjects
        } else if array_count == total {
            ArrayType::AllArrays
        } else if primitive_count == total {
            ArrayType::AllPrimitives
        } else {
            ArrayType::Mixed
        };

        ArrayAnalysis {
            array_type,
            total_elements: total,
            null_count,
            bool_count,
            number_count,
            string_count,
            array_count,
            object_count,
        }
    }

    /// Format array of primitives (numbers, strings, booleans, nulls)
    fn format_primitive_array(&mut self, array: &[Value]) -> FormattingResult<String> {
        let mut values = Vec::with_capacity(array.len());

        for value in array {
            let formatted = self.format_primitive_value(value)?;
            values.push(formatted);
        }

        let delimiter = self.config.delimiter.as_str();

        if self.config.pretty {
            let mut result = String::new();
            result.push('[');

            for (i, value_str) in values.iter().enumerate() {
                if i > 0 {
                    result.push_str(delimiter);
                    result.push(' ');
                }
                result.push_str(value_str);
            }

            result.push(']');
            Ok(result)
        } else {
            Ok(format!("[{}]", values.join(delimiter)))
        }
    }

    /// Format array of objects
    fn format_object_array(&mut self, array: &[Value]) -> FormattingResult<String> {
        let mut result = String::new();
        result.push('[');

        if self.config.pretty {
            result.push('\n');
        }

        for (i, value) in array.iter().enumerate() {
            if i > 0 {
                result.push(',');
                if self.config.pretty {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
            }

            if self.config.pretty {
                self.indent_level += 1;
                result.push_str(&self.get_indent());
            }

            let formatted = self.format_complex_value(value)?;
            result.push_str(&formatted);

            if self.config.pretty {
                self.indent_level -= 1;
            }
        }

        if self.config.pretty {
            result.push('\n');
            result.push_str(&self.get_indent());
        }

        result.push(']');
        Ok(result)
    }

    /// Format array of arrays
    fn format_nested_arrays(&mut self, array: &[Value]) -> FormattingResult<String> {
        let mut result = String::new();
        result.push('[');

        for (i, value) in array.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }

            let formatted = if let Value::Array(inner) = value {
                self.format_mixed_array(inner)?
            } else {
                return Err(FormattingError::invalid_structure(
                    "Expected array in nested array".to_string(),
                ));
            };

            result.push_str(&formatted);
        }

        result.push(']');
        Ok(result)
    }

    /// Format truly mixed array with different types
    fn format_truly_mixed_array(&mut self, array: &[Value]) -> FormattingResult<String> {
        let mut result = String::new();
        result.push('[');

        if self.config.pretty {
            result.push('\n');
        }

        for (i, value) in array.iter().enumerate() {
            if i > 0 {
                result.push(',');
                if self.config.pretty {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
            }

            if self.config.pretty {
                self.indent_level += 1;
                result.push_str(&self.get_indent());
            }

            // Format based on value type
            let formatted = if value.is_object() || value.is_array() {
                self.format_complex_value(value)?
            } else {
                self.format_primitive_value(value)?
            };

            result.push_str(&formatted);

            if self.config.pretty {
                self.indent_level -= 1;
            }
        }

        if self.config.pretty {
            result.push('\n');
            result.push_str(&self.get_indent());
        }

        result.push(']');
        Ok(result)
    }

    /// Format a primitive value
    fn format_primitive_value(&self, value: &Value) -> FormattingResult<String> {
        match value {
            Value::Null => Ok("null".to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Number(n) => self.format_number(n),
            Value::String(s) => self.format_string(s),
            _ => Err(FormattingError::invalid_structure(
                "Expected primitive value".to_string(),
            )),
        }
    }

    /// Format a complex value (object or array)
    fn format_complex_value(&mut self, value: &Value) -> FormattingResult<String> {
        match value {
            Value::Object(obj) => {
                // Simple inline object formatting for mixed arrays
                let mut result = String::new();
                result.push('{');

                let mut first = true;
                for (key, val) in obj {
                    if !first {
                        result.push(',');
                        result.push(' ');
                    }
                    first = false;

                    result.push_str(key);
                    result.push(':');
                    result.push(' ');

                    let formatted = self.format_primitive_value(val)?;
                    result.push_str(&formatted);
                }

                result.push('}');
                Ok(result)
            }
            Value::Array(arr) => self.format_mixed_array(arr),
            _ => self.format_primitive_value(value),
        }
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
        use super::quotes::QuoteEngine;
        use crate::conversion::QuoteStrategy;

        let engine = QuoteEngine::new(self.config.delimiter.as_str().to_string());

        match self.config.quote_strings {
            QuoteStrategy::Always => engine.quote(value),
            QuoteStrategy::Never => Ok(value.to_string()),
            QuoteStrategy::Smart => engine.format(value),
        }
    }

    /// Get current indentation string
    fn get_indent(&self) -> String {
        " ".repeat(self.indent_level * self.config.indent_size as usize)
    }
}

/// Array type classification
#[derive(Debug, PartialEq)]
enum ArrayType {
    AllPrimitives,
    AllObjects,
    AllArrays,
    Mixed,
}

/// Array content analysis result
#[derive(Debug)]
struct ArrayAnalysis {
    array_type: ArrayType,
    total_elements: usize,
    null_count: usize,
    bool_count: usize,
    number_count: usize,
    string_count: usize,
    array_count: usize,
    object_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_all_primitives_array() {
        let config = ConversionConfig::default();
        let mut formatter = MixedArrayFormatter::new(&config);

        let array = json!([1, 2, 3, "hello", true, null]);
        let result = formatter
            .format_mixed_array(array.as_array().unwrap())
            .unwrap();

        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_all_objects_array() {
        let config = ConversionConfig::default();
        let mut formatter = MixedArrayFormatter::new(&config);

        let array = json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]);

        let result = formatter
            .format_mixed_array(array.as_array().unwrap())
            .unwrap();
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
    }

    #[test]
    fn test_nested_arrays() {
        let config = ConversionConfig::default();
        let mut formatter = MixedArrayFormatter::new(&config);

        let array = json!([[1, 2], [3, 4], [5, 6]]);
        let result = formatter
            .format_mixed_array(array.as_array().unwrap())
            .unwrap();

        assert!(result.starts_with('['));
        assert!(result.contains("[1"));
        assert!(result.contains("[3"));
    }

    #[test]
    fn test_truly_mixed_array() {
        let config = ConversionConfig::default();
        let mut formatter = MixedArrayFormatter::new(&config);

        let array = json!([
            42,
            "hello",
            {"key": "value"},
            [1, 2, 3],
            true,
            null
        ]);

        let result = formatter
            .format_mixed_array(array.as_array().unwrap())
            .unwrap();
        assert!(result.contains("42"));
        assert!(result.contains("hello"));
        assert!(result.contains("key"));
        assert!(result.contains("true"));
        assert!(result.contains("null"));
    }

    #[test]
    fn test_empty_array() {
        let config = ConversionConfig::default();
        let mut formatter = MixedArrayFormatter::new(&config);

        let array = json!([]);
        let result = formatter
            .format_mixed_array(array.as_array().unwrap())
            .unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_array_analysis() {
        let config = ConversionConfig::default();
        let formatter = MixedArrayFormatter::new(&config);

        // All primitives
        let array = json!([1, 2, "hello", true]);
        let analysis = formatter.analyze_array(array.as_array().unwrap());
        assert_eq!(analysis.array_type, ArrayType::AllPrimitives);
        assert_eq!(analysis.total_elements, 4);

        // All objects
        let array = json!([{}, {"a": 1}]);
        let analysis = formatter.analyze_array(array.as_array().unwrap());
        assert_eq!(analysis.array_type, ArrayType::AllObjects);

        // Mixed
        let array = json!([1, {}, []]);
        let analysis = formatter.analyze_array(array.as_array().unwrap());
        assert_eq!(analysis.array_type, ArrayType::Mixed);
    }

    #[test]
    fn test_pretty_vs_compact() {
        let config = ConversionConfig {
            pretty: true,
            ..Default::default()
        };
        let mut formatter = MixedArrayFormatter::new(&config);

        let array = json!([1, 2, 3]);
        let pretty_result = formatter
            .format_mixed_array(array.as_array().unwrap())
            .unwrap();

        let config = ConversionConfig {
            pretty: false,
            ..Default::default()
        };
        let mut formatter = MixedArrayFormatter::new(&config);
        let compact_result = formatter
            .format_mixed_array(array.as_array().unwrap())
            .unwrap();

        assert!(pretty_result.len() >= compact_result.len());
    }
}
