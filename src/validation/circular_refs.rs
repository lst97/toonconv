//! Circular reference detection for JSON structures
//!
//! Prevents infinite loops and stack overflows when processing
//! JSON structures that may contain circular references.

use crate::error::{FormattingError, FormattingResult};
use serde_json::Value;
use std::collections::HashSet;

/// Circular reference detector
pub struct CircularRefDetector {
    /// Maximum depth to traverse before assuming circular reference
    max_depth: usize,

    /// Track visited object/array paths
    visited_paths: HashSet<String>,
}

impl CircularRefDetector {
    /// Create a new circular reference detector
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            visited_paths: HashSet::new(),
        }
    }

    /// Detect circular references in a JSON value
    pub fn detect(&mut self, value: &Value) -> FormattingResult<()> {
        self.visited_paths.clear();
        self.detect_recursive(value, 0, String::new())
    }

    /// Recursive detection with path tracking
    fn detect_recursive(
        &mut self,
        value: &Value,
        depth: usize,
        path: String,
    ) -> FormattingResult<()> {
        // Check depth limit
        if depth > self.max_depth {
            return Err(CircularRefError::max_depth_exceeded(depth, self.max_depth).into());
        }

        // Check if we've seen this path before
        if !path.is_empty() && self.visited_paths.contains(&path) {
            return Err(CircularRefError::circular_reference_detected(path).into());
        }

        // Add current path to visited set
        if !path.is_empty() {
            self.visited_paths.insert(path.clone());
        }

        // Traverse structure
        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    self.detect_recursive(val, depth + 1, new_path)?;
                }
            }
            Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, index);
                    self.detect_recursive(val, depth + 1, new_path)?;
                }
            }
            _ => {
                // Primitives can't have circular references
            }
        }

        Ok(())
    }

    /// Check if a value would cause circular reference issues
    pub fn is_safe(&mut self, value: &Value) -> bool {
        self.detect(value).is_ok()
    }

    /// Get the maximum depth encountered
    pub fn max_depth_limit(&self) -> usize {
        self.max_depth
    }

    /// Reset the detector for reuse
    pub fn reset(&mut self) {
        self.visited_paths.clear();
    }
}

/// Circular reference error
#[derive(Debug, Clone)]
pub struct CircularRefError {
    pub kind: CircularRefErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircularRefErrorKind {
    CircularReference,
    MaxDepthExceeded,
}

impl CircularRefError {
    /// Create a circular reference error
    pub fn circular_reference_detected(path: String) -> Self {
        Self {
            kind: CircularRefErrorKind::CircularReference,
            message: format!("Circular reference detected at path: {}", path),
        }
    }

    /// Create a max depth exceeded error
    pub fn max_depth_exceeded(current: usize, max: usize) -> Self {
        Self {
            kind: CircularRefErrorKind::MaxDepthExceeded,
            message: format!(
                "Maximum nesting depth ({}) exceeded at depth {}",
                max, current
            ),
        }
    }
}

impl std::fmt::Display for CircularRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CircularRefError {}

impl From<CircularRefError> for FormattingError {
    fn from(err: CircularRefError) -> Self {
        FormattingError::invalid_structure(err.message)
    }
}

/// Convenience function to check for circular references
pub fn has_circular_refs(value: &Value, max_depth: usize) -> bool {
    let mut detector = CircularRefDetector::new(max_depth);
    !detector.is_safe(value)
}

/// Convenience function to validate depth
pub fn validate_depth(value: &Value, max_depth: usize) -> FormattingResult<()> {
    let mut detector = CircularRefDetector::new(max_depth);
    detector.detect(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_object_no_circular_ref() {
        let mut detector = CircularRefDetector::new(100);
        let json = json!({"name": "Alice", "age": 30});

        assert!(detector.detect(&json).is_ok());
        assert!(detector.is_safe(&json));
    }

    #[test]
    fn test_nested_structure_no_circular_ref() {
        let mut detector = CircularRefDetector::new(100);
        let json = json!({
            "user": {
                "profile": {
                    "name": "Alice",
                    "address": {
                        "city": "NYC"
                    }
                }
            }
        });

        assert!(detector.detect(&json).is_ok());
    }

    #[test]
    fn test_array_structure_no_circular_ref() {
        let mut detector = CircularRefDetector::new(100);
        let json = json!([
            {"id": 1, "data": [1, 2, 3]},
            {"id": 2, "data": [4, 5, 6]}
        ]);

        assert!(detector.detect(&json).is_ok());
    }

    #[test]
    fn test_max_depth_exceeded() {
        let mut detector = CircularRefDetector::new(3);

        // Create a structure nested 5 levels deep
        let json = json!({
            "l1": {
                "l2": {
                    "l3": {
                        "l4": {
                            "l5": "too deep"
                        }
                    }
                }
            }
        });

        let result = detector.detect(&json);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("depth"));
    }

    #[test]
    fn test_primitives_always_safe() {
        let mut detector = CircularRefDetector::new(10);

        assert!(detector.detect(&json!(null)).is_ok());
        assert!(detector.detect(&json!(true)).is_ok());
        assert!(detector.detect(&json!(42)).is_ok());
        assert!(detector.detect(&json!("hello")).is_ok());
    }

    #[test]
    fn test_empty_structures_safe() {
        let mut detector = CircularRefDetector::new(10);

        assert!(detector.detect(&json!({})).is_ok());
        assert!(detector.detect(&json!([])).is_ok());
    }

    #[test]
    fn test_detector_reset() {
        let mut detector = CircularRefDetector::new(100);

        let json1 = json!({"a": {"b": "c"}});
        assert!(detector.detect(&json1).is_ok());

        // Reset and use again
        detector.reset();

        let json2 = json!({"x": {"y": "z"}});
        assert!(detector.detect(&json2).is_ok());
    }

    #[test]
    fn test_complex_nested_arrays() {
        let mut detector = CircularRefDetector::new(50);

        let json = json!([[[1, 2], [3, 4]], [[5, 6], [7, 8]], [[9, 10], [11, 12]]]);

        assert!(detector.detect(&json).is_ok());
    }

    #[test]
    fn test_convenience_functions() {
        let json = json!({"valid": "structure"});

        assert!(!has_circular_refs(&json, 100));
        assert!(validate_depth(&json, 100).is_ok());
    }

    #[test]
    fn test_convenience_functions_with_deep_structure() {
        // Create deeply nested structure
        let mut json = json!({"deepest": "value"});
        for i in (1..=10).rev() {
            json = json!({format!("level{}", i): json});
        }

        // Should be fine with high depth limit
        assert!(!has_circular_refs(&json, 100));

        // Should fail with low depth limit
        assert!(validate_depth(&json, 5).is_err());
    }

    #[test]
    fn test_error_messages() {
        let err1 = CircularRefError::circular_reference_detected("user.profile".to_string());
        assert!(err1.to_string().contains("Circular reference"));
        assert!(err1.to_string().contains("user.profile"));

        let err2 = CircularRefError::max_depth_exceeded(10, 5);
        assert!(err2.to_string().contains("Maximum nesting depth"));
        assert!(err2.to_string().contains("5"));
    }

    #[test]
    fn test_path_tracking() {
        let mut detector = CircularRefDetector::new(100);

        let json = json!({
            "users": [
                {"name": "Alice", "friends": [{"name": "Bob"}]},
                {"name": "Charlie", "friends": []}
            ]
        });

        assert!(detector.detect(&json).is_ok());

        // Visited paths should include various structure paths
        assert!(!detector.visited_paths.is_empty());
    }
}
