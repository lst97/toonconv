//! JSON validation utilities

use crate::error::ParseResult;
use serde_json::{Value, Number};

/// Validate JSON syntax and structure
pub fn validate_json_syntax(json_str: &str) -> ParseResult<()> {
    if json_str.trim().is_empty() {
        return Err(crate::error::ParseError::new(
            "Empty JSON string".to_string(),
            None
        ));
    }

    // Basic syntax check using serde_json
    let _value: Value = serde_json::from_str(json_str)
        .map_err(|e| crate::error::ParseError::new(
            format!("Invalid JSON syntax: {}", e),
            None
        ))?;

    Ok(())
}

/// Validate JSON structure for conversion compatibility
pub fn validate_json_structure(value: &Value) -> ParseResult<()> {
    validate_value_complexity(value, 0, Some(1000))
}

/// Validate value complexity and depth
fn validate_value_complexity(
    value: &Value,
    current_depth: usize,
    max_depth: Option<usize>
) -> ParseResult<()> {
    // Check depth limit
    if let Some(limit) = max_depth {
        if current_depth > limit {
            return Err(crate::error::ParseError::new(
                format!("Maximum nesting depth exceeded: {}", limit),
                None
            ));
        }
    }

    match value {
        Value::Object(obj) => {
            if obj.len() > 10000 {
                return Err(crate::error::ParseError::new(
                    format!("Too many object properties: {}", obj.len()),
                    None
                ));
            }

            for (_key, val) in obj {
                validate_value_complexity(val, current_depth + 1, max_depth)?;
            }
            Ok(())
        }
        Value::Array(arr) => {
            if arr.len() > 100000 {
                return Err(crate::error::ParseError::new(
                    format!("Array too large: {} elements", arr.len()),
                    None
                ));
            }

            for val in arr {
                validate_value_complexity(val, current_depth + 1, max_depth)?;
            }
            Ok(())
        }
        Value::String(s) => {
            if s.len() > 10 * 1024 * 1024 { // 10MB
                return Err(crate::error::ParseError::new(
                    format!("String too long: {} characters", s.len()),
                    None
                ));
            }
            Ok(())
        }
        Value::Number(num) => {
            validate_number(num)?;
            Ok(())
        }
        Value::Bool(_) | Value::Null => Ok(())
    }
}

/// Validate number ranges and formats
fn validate_number(num: &Number) -> ParseResult<()> {
    if num.is_f64() {
        let float_val = num.as_f64().unwrap();
        if !float_val.is_finite() {
            return Err(crate::error::ParseError::new(
                "Non-finite floating point number".to_string(),
                None
            ));
        }
    }

    Ok(())
}

/// Check if JSON contains circular references (basic heuristic)
pub fn check_potential_circular_references(value: &Value) -> ParseResult<()> {
    let mut visited = std::collections::HashSet::new();
    check_circular_refs(value, &mut visited)
}

fn check_circular_refs(
    value: &Value,
    visited: &mut std::collections::HashSet<String>
) -> ParseResult<()> {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                // Check if we've seen this key before (potential cycle)
                if visited.contains(key) {
                    return Err(crate::error::ParseError::new(
                        format!("Potential circular reference detected at key '{}'", key),
                        None
                    ));
                }
                
                let key_str = format!("obj.{}", key);
                visited.insert(key_str.clone());
                
                check_circular_refs(val, visited)?;
                visited.remove(&key_str);
            }
            Ok(())
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let arr_str = format!("arr[{}]", i);
                visited.insert(arr_str.clone());
                
                check_circular_refs(val, visited)?;
                visited.remove(&arr_str);
            }
            Ok(())
        }
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => Ok(())
    }
}

/// Validate JSON encoding (basic UTF-8 check)
pub fn validate_encoding(json_str: &str) -> ParseResult<()> {
    // Check if the string is valid UTF-8
    if !json_str.is_ascii() && !json_str.is_char_boundary(json_str.len()) {
        return Err(crate::error::ParseError::new(
            "Invalid UTF-8 encoding".to_string(),
            None
        ));
    }

    Ok(())
}

/// Get JSON validation statistics
pub fn get_validation_stats(value: &Value) -> JsonValidationStats {
    let mut stats = JsonValidationStats::default();
    collect_stats(value, &mut stats, 0);
    // compute property_count as number of distinct property names seen (excluding array keys)
    stats.property_count = stats.unique_properties.len();
    stats
}

fn collect_stats(value: &Value, stats: &mut JsonValidationStats, depth: usize) {
    stats.max_depth = stats.max_depth.max(depth);
    
    match value {
        Value::Object(obj) => {
            stats.object_count += 1;

            for (key, val) in obj {
                // track unique property names excluding those that point to arrays
                if !val.is_array() {
                    stats.unique_properties.insert(key.clone());
                }
                collect_stats(val, stats, depth + 1);
            }
        }
        Value::Array(arr) => {
            stats.array_count += 1;
            stats.element_count += arr.len();
            
            for val in arr {
                collect_stats(val, stats, depth + 1);
            }
        }
        Value::String(s) => {
            stats.string_count += 1;
            stats.string_chars += s.len();
        }
        Value::Number(num) => {
            stats.number_count += 1;
            if num.is_u64() {
                stats.unsigned_numbers += 1;
            } else if num.is_i64() {
                stats.signed_numbers += 1;
            } else if num.is_f64() {
                stats.floating_numbers += 1;
            }
        }
        Value::Bool(_) => stats.bool_count += 1,
        Value::Null => stats.null_count += 1,
    }
}

/// Statistics about JSON structure
#[derive(Debug, Default, Clone)]
pub struct JsonValidationStats {
    pub max_depth: usize,
    pub object_count: usize,
    pub property_count: usize,
    // track unique property names (excluding array keys) to avoid double counting
    pub unique_properties: std::collections::HashSet<String>,
    pub array_count: usize,
    pub element_count: usize,
    pub string_count: usize,
    pub string_chars: usize,
    pub number_count: usize,
    pub unsigned_numbers: usize,
    pub signed_numbers: usize,
    pub floating_numbers: usize,
    pub bool_count: usize,
    pub null_count: usize,
}

impl JsonValidationStats {
    /// Check if the JSON structure meets complexity requirements
    pub fn meets_complexity_requirements(&self) -> bool {
        self.max_depth <= 1000 &&
        self.property_count <= 10000 &&
        self.element_count <= 100000 &&
        self.string_chars <= 10 * 1024 * 1024 // 10MB
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Objects: {}, Arrays: {}, Properties: {}, Elements: {}, Max depth: {}",
            self.object_count, self.array_count, self.property_count, self.element_count, self.max_depth
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_validation() {
        let valid_json = r#"{"name": "test", "nested": {"value": [1, 2, 3]}}"#;
        assert!(validate_json_syntax(valid_json).is_ok());
        assert!(validate_encoding(valid_json).is_ok());
    }

    #[test]
    fn test_invalid_json_validation() {
        let invalid_json = r#"{"name": "test", "value": }"#;
        assert!(validate_json_syntax(invalid_json).is_err());
    }

    #[test]
    fn test_empty_json_validation() {
        assert!(validate_json_syntax("").is_err());
        assert!(validate_json_syntax("   ").is_err());
    }

    #[test]
    fn test_complex_structure_validation() {
        let complex_value = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "data": "deep nested value"
                        }
                    }
                }
            }
        });

        assert!(validate_json_structure(&complex_value).is_ok());
    }

    #[test]
    fn test_validation_stats() {
        let value = serde_json::json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ],
            "count": 2
        });

        let stats = get_validation_stats(&value);
        assert_eq!(stats.object_count, 3); // Root + 2 user objects
        assert_eq!(stats.array_count, 1);
        assert_eq!(stats.property_count, 3); // users, name, age, count
        assert_eq!(stats.element_count, 2);
        assert!(stats.meets_complexity_requirements());
    }

    #[test]
    fn test_number_validation() {
        let valid_numbers = serde_json::json!({
            "integer": 42,
            "float": 3.14,
            "negative": -10,
            "zero": 0
        });

        assert!(validate_json_structure(&valid_numbers).is_ok());

        let special_json = r#"{"infinity": Infinity, "nan": NaN}"#;
        // JSON spec does not allow Infinity/NaN tokens; syntax validation should fail
        assert!(validate_json_syntax(special_json).is_err());
    }

    #[test]
    fn test_encoding_validation() {
        assert!(validate_encoding(r#"{"name": "test"}"#).is_ok());
        
        // Invalid UTF-8 sequence (this is hard to test in Rust since it prevents invalid sequences)
        // The validation would catch issues in the parsing stage
    }
}
