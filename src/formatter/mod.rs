//! TOON formatting module

pub mod mixed_arrays;
pub mod nested;
pub mod quotes;
pub mod schema;

use crate::conversion::{ConversionConfig, ConversionResult};
use crate::error::{FormattingError, FormattingResult};
use serde_json::{Map, Value};

/// Main TOON formatter
pub struct ToonFormatter {
    config: ConversionConfig,
    indent_level: usize,
    in_tabular_array: bool,
}

impl ToonFormatter {
    /// Create a new formatter with configuration
    pub fn new(config: ConversionConfig) -> Self {
        Self {
            config,
            indent_level: 0,
            in_tabular_array: false,
        }
    }

    /// Format a JSON value as TOON
    pub fn format(&mut self, value: &Value) -> ConversionResult<String> {
        let result = match value {
            Value::Null => self.format_null(),
            Value::Bool(b) => self.format_bool(*b),
            Value::Number(n) => self.format_number(n),
            Value::String(s) => self.format_string(s),
            Value::Array(a) => self.format_array(a),
            Value::Object(o) => self.format_object(o),
        };

        match result {
            Ok(output) => {
                if self.config.validate_output {
                    self.validate_output(&output)?;
                }
                Ok(output)
            }
            Err(e) => Err(crate::error::ConversionError::FormattingError(e)),
        }
    }

    /// Format a null value
    fn format_null(&self) -> FormattingResult<String> {
        Ok("null".to_string())
    }

    /// Format a boolean value
    fn format_bool(&self, value: bool) -> FormattingResult<String> {
        Ok(value.to_string())
    }

    /// Format a number value
    fn format_number(&self, value: &serde_json::Number) -> FormattingResult<String> {
        // Check if it's an integer
        if value.is_i64() || value.is_u64() {
            return Ok(value.to_string());
        }

        // It's a float
        if let Some(f) = value.as_f64() {
            if f.is_infinite() || f.is_nan() {
                return Err(FormattingError::invalid_structure(
                    "Invalid number: infinity or NaN not supported in TOON".to_string(),
                ));
            }

            // TOON spec: use minimal representation without trailing zeros
            // Check if it's a whole number
            if f.fract() == 0.0 {
                return Ok(format!("{}", f as i64));
            }

            // Format with enough precision then remove trailing zeros
            let formatted = format!("{}", f);

            // If the number has a decimal point, clean up trailing zeros
            if formatted.contains('.') {
                let trimmed = formatted.trim_end_matches('0');
                // If we removed all decimals, remove the dot too
                if trimmed.ends_with('.') {
                    Ok(trimmed.trim_end_matches('.').to_string())
                } else {
                    Ok(trimmed.to_string())
                }
            } else {
                Ok(formatted)
            }
        } else {
            Ok(value.to_string())
        }
    }

    /// Format a string value
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
        if value.starts_with(' ') || value.ends_with(' ') {
            return true;
        }

        // Starts with "- " (looks like list item marker)
        if value.starts_with("- ") {
            return true;
        }

        // Structural characters
        let structural_chars = ":[]{}";
        if value.chars().any(|c| structural_chars.contains(c)) {
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

    /// Check if a key needs quoting according to TOON rules
    /// Keys need quoting if they contain: colon, spaces, or are numeric
    fn should_quote_key(&self, key: &str) -> bool {
        if key.is_empty() {
            return true;
        }

        // Contains colon (structural character)
        if key.contains(':') {
            return true;
        }

        // Contains spaces
        if key.contains(' ') {
            return true;
        }

        // Is numeric (starts with digit or is a valid number)
        if key.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return true;
        }

        // Contains other structural characters
        if key.chars().any(|c| matches!(c, '[' | ']' | '{' | '}' | ',')) {
            return true;
        }

        false
    }

    /// Format a key with quoting if needed
    fn format_key(&self, key: &str) -> FormattingResult<String> {
        if self.should_quote_key(key) {
            self.quote_string(key)
        } else {
            Ok(key.to_string())
        }
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

    /// Format an array
    fn format_array(&mut self, array: &[Value]) -> FormattingResult<String> {
        if array.is_empty() {
            return Ok("[0]:".to_string());
        }

        // Check if this is a uniform array of objects (tabular format)
        if self.is_uniform_object_array(array) {
            return self.format_tabular_array(array);
        }

        // Check if this is a uniform array of primitives
        if self.is_uniform_primitive_array(array) {
            return self.format_primitive_array(array);
        }

        // Mixed or complex array
        // Increment indent so items have proper indentation
        self.indent_level += 1;
        let result = self.format_mixed_array(array);
        self.indent_level -= 1;
        result
    }

    /// Check if array contains uniform objects (truly uniform for tabular format)
    fn is_uniform_object_array(&self, array: &[Value]) -> bool {
        if array.is_empty() {
            return false;
        }

        // All elements must be objects
        if !array.iter().all(|v| v.is_object()) {
            return false;
        }

        // All objects must have the same keys
        let first_obj = array[0].as_object().unwrap();
        let first_keys: std::collections::HashSet<&str> =
            first_obj.keys().map(|k| k.as_str()).collect();

        // Check if all objects have same keys
        if !array.iter().all(|v| {
            let keys: std::collections::HashSet<&str> =
                v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
            keys == first_keys
        }) {
            return false;
        }

        // Additional check: if any value is an array or object, verify they're uniform too
        // This prevents using tabular format for objects with nested complex structures
        for key in first_keys {
            let first_val = first_obj.get(key).unwrap();

            // If the value is an array, check if all corresponding arrays are identical in structure
            if first_val.is_array() {
                let first_arr = first_val.as_array().unwrap();
                let first_len = first_arr.len();

                // Check if all objects have arrays of the same length for this key
                for obj in array.iter().skip(1) {
                    let val = obj.as_object().unwrap().get(key).unwrap();
                    if !val.is_array() || val.as_array().unwrap().len() != first_len {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if array contains uniform primitives
    fn is_uniform_primitive_array(&self, array: &[Value]) -> bool {
        if array.is_empty() {
            return false;
        }

        // Primitives are: null, bool, number, string (NOT objects or arrays)
        array.iter().all(|v| !v.is_object() && !v.is_array())
    }

    /// Format primitive array (TOON inline format: [count]: val1,val2,val3)
    fn format_primitive_array(&mut self, array: &[Value]) -> FormattingResult<String> {
        let length = array.len();

        let mut values = Vec::with_capacity(length);
        for value in array {
            let formatted = self.format_value(value)?;
            values.push(formatted);
        }

        let delimiter = self.config.delimiter.as_str();
        let values_str = values.join(delimiter);

        // TOON format: [count]: val1,val2,val3 (inline with count)
        Ok(format!("[{}]: {}", length, values_str))
    }

    /// Format tabular array (uniform objects) - TOON format: [count]{field1,field2}:
    fn format_tabular_array(&mut self, array: &[Value]) -> FormattingResult<String> {
        if array.is_empty() {
            return Ok("[0]:".to_string());
        }

        let first_obj = array[0].as_object().unwrap();
        let fields: Vec<&str> = first_obj.keys().map(|k| k.as_str()).collect();

        // Build TOON schema declaration: [count]{field1,field2}:
        // Quote field names that need quoting
        let quoted_fields: Vec<String> = fields
            .iter()
            .map(|f| self.format_key(f).unwrap_or_else(|_| f.to_string()))
            .collect();
        let field_list = quoted_fields.join(",");
        let schema = format!("[{}]{{{}}}", array.len(), field_list);

        let delimiter = self.config.delimiter.as_str();
        let mut result = String::new();

        result.push_str(&schema);
        result.push(':');
        result.push('\n');

        // Set flag to indicate we're in a tabular array (for number formatting)
        self.in_tabular_array = true;

        // Increment indent for the data rows
        self.indent_level += 1;

        // Format each row with proper indentation
        for (i, obj) in array.iter().enumerate() {
            // Indent the row one level deeper than the schema header
            result.push_str(&self.indent());

            let obj_map = obj.as_object().unwrap();
            let mut row_values = Vec::with_capacity(fields.len());

            for field in &fields {
                let value = obj_map.get(&field.to_string()).unwrap();
                let formatted = self.format_value(value)?;
                row_values.push(formatted);
            }

            let row_str = row_values.join(delimiter);
            result.push_str(&row_str);

            if i < array.len() - 1 {
                result.push('\n');
            }
        }

        // Reset indent level and flag after formatting tabular array
        self.indent_level -= 1;
        self.in_tabular_array = false;

        Ok(result)
    }

    /// Format mixed array (TOON dash-prefix format)
    fn format_mixed_array(&mut self, array: &[Value]) -> FormattingResult<String> {

        // TOON format for non-uniform arrays: use dash prefix with count
        let mut result = String::new();
        result.push_str(&format!("[{}]:", array.len()));

        // Note: caller handles base indentation, we just add one level for array items
        for (_i, value) in array.iter().enumerate() {
            result.push('\n');
            result.push_str(&self.indent());

            // For objects in mixed arrays: dash then newline, content indented
            if value.is_object() {
                let obj = value.as_object().unwrap();
                let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

                // Objects: dash followed by newline (no space)
                result.push('-');
                result.push('\n');

                // Increment indent for object content
                self.indent_level += 1;
                result.push_str(&self.indent());

                for (j, key) in keys.iter().enumerate() {
                    if j > 0 {
                        result.push('\n');
                        result.push_str(&self.indent());
                    }

                    let formatted_key = self.format_key(key)?;
                    result.push_str(&formatted_key);

                    let val = obj.get(*key).unwrap();

                    // Handle arrays specially - inline primitive arrays
                    if val.is_array() {
                        let arr = val.as_array().unwrap();
                        if self.is_uniform_primitive_array(arr) {
                            // Inline format: methods[2]: GET,POST
                            let formatted = self.format_primitive_array(arr)?;
                            result.push_str(&formatted);
                        } else {
                            // Complex array - format as nested structure
                            result.push_str(&format!("[{}]:", arr.len()));
                            result.push('\n');
                            self.indent_level += 1;

                            // Format each element with proper indentation
                            for item in arr.iter() {
                                result.push_str(&self.indent());
                                result.push_str("- ");

                                if item.is_object() {
                                    // Format object fields inline after the dash
                                    let obj_inner = item.as_object().unwrap();
                                    let inner_keys: Vec<&str> =
                                        obj_inner.keys().map(|k| k.as_str()).collect();

                                    for (k, inner_key) in inner_keys.iter().enumerate() {
                                        if k > 0 {
                                            result.push('\n');
                                            result.push_str(&self.indent());
                                            result.push_str("  ");
                                        }

                                        result.push_str(inner_key);

                                        let inner_val = obj_inner.get(*inner_key).unwrap();

                                        // For nested values in deeply nested structures
                                        if inner_val.is_array() {
                                            let inner_arr = inner_val.as_array().unwrap();
                                            if self.is_uniform_primitive_array(inner_arr) {
                                                let formatted =
                                                    self.format_primitive_array(inner_arr)?;
                                                result.push_str(&formatted);
                                            } else {
                                                // Recursively format complex nested arrays with dash-prefix
                                                // Add array count directly after key: medications[2]:
                                                result.push_str(&format!("[{}]:", inner_arr.len()));
                                                result.push('\n');
                                                self.indent_level += 1;

                                                // Format each item with dash-prefix
                                                for item in inner_arr.iter() {
                                                    result.push_str(&self.indent());
                                                    result.push_str("- ");

                                                    if item.is_object() {
                                                        let obj_deep = item.as_object().unwrap();
                                                        let deep_keys: Vec<&str> = obj_deep
                                                            .keys()
                                                            .map(|k| k.as_str())
                                                            .collect();

                                                        for (dk, deep_key) in
                                                            deep_keys.iter().enumerate()
                                                        {
                                                            if dk > 0 {
                                                                result.push('\n');
                                                                result.push_str(&self.indent());
                                                                result.push_str("  ");
                                                            }

                                                            result.push_str(deep_key);
                                                            result.push_str(": ");

                                                            let deep_val =
                                                                obj_deep.get(*deep_key).unwrap();
                                                            let formatted_deep =
                                                                self.format_value(deep_val)?;
                                                            result.push_str(&formatted_deep);
                                                        }
                                                    } else {
                                                        let formatted_item =
                                                            self.format_value(item)?;
                                                        result.push_str(&formatted_item);
                                                    }
                                                    result.push('\n');
                                                }

                                                self.indent_level -= 1;
                                                // Remove trailing newline
                                                if result.ends_with('\n') {
                                                    result.pop();
                                                }
                                            }
                                        } else if inner_val.is_object() {
                                            // Format nested object
                                            result.push_str(":");
                                            result.push('\n');
                                            self.indent_level += 1;
                                            let formatted =
                                                self.format_object(inner_val.as_object().unwrap())?;
                                            result.push_str(&formatted);
                                            self.indent_level -= 1;
                                        } else {
                                            result.push_str(": ");
                                            let formatted_val = self.format_value(inner_val)?;
                                            result.push_str(&formatted_val);
                                        }
                                    }
                                } else {
                                    let formatted = self.format_value(item)?;
                                    result.push_str(&formatted);
                                }
                                result.push('\n');
                            }

                            self.indent_level -= 1;
                            // Remove trailing newline
                            if result.ends_with('\n') {
                                result.pop();
                            }
                        }
                    } else if val.is_object() {
                        // Nested object - format on new line
                        result.push(':');
                        result.push('\n');
                        let formatted = self.format_object(val.as_object().unwrap())?;
                        result.push_str(&formatted);
                    } else {
                        // Simple value - inline
                        result.push_str(": ");
                        let formatted = self.format_value(val)?;
                        result.push_str(&formatted);
                    }
                }
                // Restore indent level after object content
                self.indent_level -= 1;
            } else if value.is_array() {
                // Nested array: - [count]: ...
                result.push_str("- ");
                let formatted = self.format_value(value)?;
                result.push_str(&formatted);
            } else {
                // Primitive value: - value
                result.push_str("- ");
                let formatted = self.format_value(value)?;
                result.push_str(&formatted);
            }
        }

        Ok(result)
    }

    /// Format an object
    fn format_object(&mut self, object: &Map<String, Value>) -> FormattingResult<String> {
        if object.is_empty() {
            // TOON spec: empty object is represented as empty string
            return Ok("".to_string());
        }

        if self.config.pretty {
            self.format_pretty_object(object)
        } else {
            self.format_compact_object(object)
        }
    }

    /// Format object in pretty mode (TOON compliant - no braces)
    fn format_pretty_object(&mut self, object: &Map<String, Value>) -> FormattingResult<String> {
        let mut result = String::new();
        let keys: Vec<&str> = object.keys().map(|k| k.as_str()).collect();

        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }

            result.push_str(&self.indent());

            // Format key with quoting if needed
            let formatted_key = self.format_key(key)?;
            result.push_str(&formatted_key);

            let value = object.get(*key).unwrap();

            // Check if this is an array that should be formatted with tabular/inline format
            if value.is_array() {
                let arr = value.as_array().unwrap();

                // Check if it's a uniform object array (tabular)
                if self.is_uniform_object_array(arr) {
                    let formatted = self.format_tabular_array(arr)?;
                    result.push_str(&formatted);
                } else if self.is_uniform_primitive_array(arr) {

                    // Inline primitive array
                    let formatted = self.format_primitive_array(arr)?;
                    result.push_str(&formatted);
                } else {
                    // Mixed array with dash-prefix
                    self.indent_level += 1;
                    let formatted = self.format_mixed_array(arr)?;
                    result.push_str(&formatted);
                    self.indent_level -= 1;
                }
            } else if value.is_object() {
                // Nested object goes on next line
                result.push(':');
                result.push('\n');
                self.indent_level += 1;
                let formatted_value = self.format_value(value)?;
                result.push_str(&formatted_value);
                self.indent_level -= 1;
            } else {
                // Primitive values go on same line
                result.push(':');
                result.push(' ');
                let formatted_value = self.format_value(value)?;
                result.push_str(&formatted_value);
            }
        }

        Ok(result)
    }

    /// Format object in compact mode (TOON compliant - no braces)
    fn format_compact_object(&mut self, object: &Map<String, Value>) -> FormattingResult<String> {
        let mut result = String::new();
        let keys: Vec<&str> = object.keys().map(|k| k.as_str()).collect();

        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }

            // Format key with quoting if needed
            let formatted_key = self.format_key(key)?;
            result.push_str(&formatted_key);
            result.push(':');

            // Format value
            let value = object.get(*key).unwrap();
            let formatted_value = self.format_value(value)?;
            result.push_str(&formatted_value);
        }

        Ok(result)
    }

    /// Format a single value (used by arrays)
    fn format_value(&mut self, value: &Value) -> FormattingResult<String> {
        match value {
            Value::Null => self.format_null(),
            Value::Bool(b) => self.format_bool(*b),
            Value::Number(n) => self.format_number(n),
            Value::String(s) => self.format_string(s),
            Value::Array(a) => self.format_array(a),
            Value::Object(o) => self.format_object(o),
        }
    }

    /// Get current indentation string
    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.config.indent_size as usize)
    }

    /// Validate TOON output compliance
    fn validate_output(&self, output: &str) -> FormattingResult<()> {
        // Basic validation - ensure output doesn't have obvious issues
        // Note: Empty string is valid for empty objects per TOON spec

        // Basic bracket balance check across the entire output (not per line)
        let mut brace_count = 0;
        let mut bracket_count = 0;

        for ch in output.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                _ => {}
            }

            // If we go negative, something is wrong
            if brace_count < 0 || bracket_count < 0 {
                return Err(FormattingError::invalid_structure(format!(
                    "Unbalanced brackets in TOON output"
                )));
            }
        }

        // If we don't end at 0, brackets are unbalanced
        if brace_count != 0 || bracket_count != 0 {
            return Err(FormattingError::invalid_structure(format!(
                "Unbalanced brackets in TOON output"
            )));
        }

        Ok(())
    }
}

/// Convenience function to format JSON as TOON
pub fn format_to_toon(value: &Value, config: &ConversionConfig) -> ConversionResult<String> {
    let mut formatter = ToonFormatter::new(config.clone());
    formatter.format(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::ConversionConfig;

    #[test]
    fn test_basic_formatting() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });

        let toon = formatter.format(&json).unwrap();
        assert!(toon.contains("name:"));
        assert!(toon.contains("Alice"));
        assert!(toon.contains("age:"));
        assert!(toon.contains("30"));
    }

    #[test]
    fn test_string_quoting() {
        let mut config = ConversionConfig::default();
        config.quote_strings = crate::conversion::QuoteStrategy::Smart;
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "normal": "hello",
            "empty": "",
            "keyword": "true",
            "number": "42",
            "spaces": " hello "
        });

        let toon = formatter.format(&json).unwrap();
        assert!(toon.contains("normal: hello"));
        assert!(toon.contains("empty: \"\""));
        assert!(toon.contains("keyword: \"true\""));
        assert!(toon.contains("number: \"42\""));
        assert!(toon.contains("spaces: \" hello \""));
    }

    #[test]
    fn test_key_quoting_with_colon() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "key:with:colons": "value"
        });

        let toon = formatter.format(&json).unwrap();
        assert!(toon.contains("\"key:with:colons\":"));
    }

    #[test]
    fn test_key_quoting_with_spaces() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "key with spaces": "value"
        });

        let toon = formatter.format(&json).unwrap();
        assert!(toon.contains("\"key with spaces\":"));
    }

    #[test]
    fn test_key_quoting_numeric_start() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "123key": "value",
            "0start": "value2"
        });

        let toon = formatter.format(&json).unwrap();
        assert!(toon.contains("\"123key\":"));
        assert!(toon.contains("\"0start\":"));
    }

    #[test]
    fn test_key_quoting_brackets() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "key[0]": "value",
            "key{x}": "value2"
        });

        let toon = formatter.format(&json).unwrap();
        assert!(toon.contains("\"key[0]\":"));
        assert!(toon.contains("\"key{x}\":"));
    }

    #[test]
    fn test_normal_keys_not_quoted() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "normalKey": "value",
            "another_key": "value2",
            "key123": "value3"
        });

        let toon = formatter.format(&json).unwrap();
        // Normal keys should not be quoted
        assert!(toon.contains("normalKey:"));
        assert!(toon.contains("another_key:"));
        assert!(toon.contains("key123:"));
        // But they should not have quotes
        assert!(!toon.contains("\"normalKey\""));
        assert!(!toon.contains("\"another_key\""));
        assert!(!toon.contains("\"key123\""));
    }

    #[test]
    fn test_number_formatting_no_trailing_zeros() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "price": 120.0,
            "rate": 25.50,
            "exact": 9.99
        });

        let toon = formatter.format(&json).unwrap();
        // 120.0 should become 120
        assert!(toon.contains("price: 120"));
        assert!(!toon.contains("120.0"));
        // 25.50 should become 25.5
        assert!(toon.contains("rate: 25.5"));
        assert!(!toon.contains("25.50"));
        // 9.99 should stay 9.99
        assert!(toon.contains("exact: 9.99"));
    }

    #[test]
    fn test_empty_array_format() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "items": []
        });

        let toon = formatter.format(&json).unwrap();
        // Empty array should be [0]:
        assert!(toon.contains("items[0]:"));
    }

    #[test]
    fn test_empty_object_format() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({});

        let toon = formatter.format(&json).unwrap();
        // Empty object should be empty string
        assert_eq!(toon.trim(), "");
    }

    #[test]
    fn test_tabular_array_format() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]
        });

        let toon = formatter.format(&json).unwrap();
        // Should have tabular format with count and schema
        assert!(toon.contains("users[2]{id,name}:"));
        assert!(toon.contains("1,Alice"));
        assert!(toon.contains("2,Bob"));
    }

    #[test]
    fn test_primitive_array_inline() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "tags": ["rust", "programming", "cli"]
        });

        let toon = formatter.format(&json).unwrap();
        // Primitive array should be inline
        assert!(toon.contains("tags[3]: rust,programming,cli"));
    }

    #[test]
    fn test_mixed_array_list_format() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "items": [
                {"type": "A"},
                "string",
                123
            ]
        });

        let toon = formatter.format(&json).unwrap();
        // Mixed array should use list format with dashes
        assert!(toon.contains("items[3]:"));
        assert!(toon.contains("-\n")); // dash then newline for objects
    }

    #[test]
    fn test_should_quote_key_method() {
        let config = ConversionConfig::default();
        let formatter = ToonFormatter::new(config);

        // Keys that should be quoted
        assert!(formatter.should_quote_key(""));
        assert!(formatter.should_quote_key("key:value"));
        assert!(formatter.should_quote_key("has space"));
        assert!(formatter.should_quote_key("123start"));
        assert!(formatter.should_quote_key("has[bracket"));
        assert!(formatter.should_quote_key("has]bracket"));
        assert!(formatter.should_quote_key("has{brace"));
        assert!(formatter.should_quote_key("has}brace"));
        assert!(formatter.should_quote_key("has,comma"));

        // Keys that should NOT be quoted
        assert!(!formatter.should_quote_key("normalKey"));
        assert!(!formatter.should_quote_key("snake_case"));
        assert!(!formatter.should_quote_key("camelCase"));
        assert!(!formatter.should_quote_key("key123"));
        assert!(!formatter.should_quote_key("_underscore"));
    }

    #[test]
    fn test_integer_vs_float_formatting() {
        let config = ConversionConfig::default();
        let mut formatter = ToonFormatter::new(config);

        let json = serde_json::json!({
            "integer": 42,
            "float": 42.5,
            "whole_float": 100.0
        });

        let toon = formatter.format(&json).unwrap();
        // Integer should remain integer
        assert!(toon.contains("integer: 42"));
        // Float should keep decimal
        assert!(toon.contains("float: 42.5"));
        // Whole float should drop .0
        assert!(toon.contains("whole_float: 100"));
        assert!(!toon.contains("100.0"));
    }
}
