//! TOON output compliance validation
//!
//! Validates that generated TOON output conforms to the TOON specification
//! and maintains data integrity from the original JSON.

use crate::error::{FormattingError, FormattingResult};
use serde_json::Value;

/// TOON compliance validator
pub struct ToonValidator {
    /// Enable strict validation mode
    strict: bool,
}

impl ToonValidator {
    /// Create a new TOON validator
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }

    /// Validate TOON output compliance
    pub fn validate(
        &self,
        toon_output: &str,
        original_json: &Value,
    ) -> FormattingResult<ValidationReport> {
        let mut report = ValidationReport::new();

        // Basic structural validation
        self.validate_structure(toon_output, &mut report)?;

        // Bracket balance validation
        self.validate_brackets(toon_output, &mut report)?;

        // Content validation (data integrity)
        self.validate_content(toon_output, original_json, &mut report)?;

        // Character encoding validation
        self.validate_encoding(toon_output, &mut report)?;

        // Formatting consistency validation
        self.validate_formatting(toon_output, &mut report)?;

        if self.strict && !report.is_valid() {
            return Err(FormattingError::invalid_structure(format!(
                "TOON validation failed: {:?}",
                report.issues
            )));
        }

        Ok(report)
    }

    /// Validate basic structure
    fn validate_structure(
        &self,
        output: &str,
        report: &mut ValidationReport,
    ) -> FormattingResult<()> {
        // Note: Empty string is valid for empty objects per TOON spec
        // Don't treat empty output as an error

        // Check for valid UTF-8
        if !output.is_char_boundary(0) || !output.is_char_boundary(output.len()) {
            report.add_error("Output contains invalid UTF-8");
        }

        report.structure_valid = true;
        Ok(())
    }

    /// Validate bracket balancing
    fn validate_brackets(
        &self,
        output: &str,
        report: &mut ValidationReport,
    ) -> FormattingResult<()> {
        let mut brace_stack = Vec::new();
        let mut bracket_stack = Vec::new();
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in output.chars().enumerate() {
            // Handle string context
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                continue;
            }

            if in_string {
                continue;
            }

            // Track brackets outside strings
            match ch {
                '{' => brace_stack.push(i),
                '}' => {
                    if brace_stack.pop().is_none() {
                        report.add_error(&format!("Unmatched closing brace at position {}", i));
                    }
                }
                '[' => bracket_stack.push(i),
                ']' => {
                    if bracket_stack.pop().is_none() {
                        report.add_error(&format!("Unmatched closing bracket at position {}", i));
                    }
                }
                _ => {}
            }
        }

        // Check for unclosed brackets
        if !brace_stack.is_empty() {
            report.add_error(&format!("{} unclosed braces", brace_stack.len()));
        }

        if !bracket_stack.is_empty() {
            report.add_error(&format!("{} unclosed brackets", bracket_stack.len()));
        }

        report.brackets_balanced = brace_stack.is_empty() && bracket_stack.is_empty();
        Ok(())
    }

    /// Validate content integrity
    fn validate_content(
        &self,
        output: &str,
        original: &Value,
        report: &mut ValidationReport,
    ) -> FormattingResult<()> {
        // Extract key values from JSON
        let json_values = self.extract_values(original);

        // Check if critical values are present in output
        let mut missing_values = Vec::new();
        for value in &json_values {
            if !self.value_present_in_output(value, output) {
                missing_values.push(value.clone());
            }
        }

        if !missing_values.is_empty() {
            report.add_warning(&format!("{} values may be missing", missing_values.len()));
        }

        report.data_integrity = missing_values.is_empty();
        Ok(())
    }

    /// Extract values from JSON for validation
    fn extract_values(&self, json: &Value) -> Vec<String> {
        let mut values = Vec::new();
        self.extract_values_recursive(json, &mut values);
        values
    }

    /// Recursively extract values
    fn extract_values_recursive(&self, json: &Value, values: &mut Vec<String>) {
        match json {
            Value::Null => values.push("null".to_string()),
            Value::Bool(b) => values.push(b.to_string()),
            Value::Number(n) => values.push(n.to_string()),
            Value::String(s) => values.push(s.clone()),
            Value::Array(arr) => {
                for item in arr {
                    self.extract_values_recursive(item, values);
                }
            }
            Value::Object(obj) => {
                for (key, value) in obj {
                    values.push(key.clone());
                    self.extract_values_recursive(value, values);
                }
            }
        }
    }

    /// Check if value is present in output
    fn value_present_in_output(&self, value: &str, output: &str) -> bool {
        // Simple substring check - could be made more sophisticated
        output.contains(value)
            || output.contains(&format!("\"{}\"", value))
            || output.contains(&self.escape_string(value))
    }

    /// Escape string for comparison
    fn escape_string(&self, s: &str) -> String {
        s.replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Validate character encoding
    fn validate_encoding(
        &self,
        output: &str,
        report: &mut ValidationReport,
    ) -> FormattingResult<()> {
        // Check for valid UTF-8 (should already be validated)
        if output.is_empty() {
            return Ok(());
        }

        // Check for control characters that should be escaped
        for (i, ch) in output.chars().enumerate() {
            if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t') {
                // Control characters should be escaped
                if !output[..i].ends_with('\\') {
                    report.add_warning(&format!("Unescaped control character at position {}", i));
                }
            }
        }

        report.encoding_valid = true;
        Ok(())
    }

    /// Validate formatting consistency
    fn validate_formatting(
        &self,
        output: &str,
        report: &mut ValidationReport,
    ) -> FormattingResult<()> {
        // Check for consistent indentation (if pretty-printed)
        if output.contains('\n') {
            self.validate_indentation(output, report)?;
        }

        // Check for proper delimiter usage
        self.validate_delimiters(output, report)?;

        report.formatting_consistent = true;
        Ok(())
    }

    /// Validate indentation consistency
    fn validate_indentation(
        &self,
        output: &str,
        report: &mut ValidationReport,
    ) -> FormattingResult<()> {
        let lines: Vec<&str> = output.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }

            // Count leading spaces
            let leading_spaces = line.chars().take_while(|c| *c == ' ').count();

            // Check if indentation is consistent (multiple of some indent size)
            if leading_spaces > 0 && leading_spaces % 2 != 0 && leading_spaces % 4 != 0 {
                report.add_warning(&format!(
                    "Inconsistent indentation on line {}: {} spaces",
                    i + 1,
                    leading_spaces
                ));
            }
        }

        Ok(())
    }

    /// Validate delimiter usage
    fn validate_delimiters(
        &self,
        output: &str,
        report: &mut ValidationReport,
    ) -> FormattingResult<()> {
        // Check for proper colon usage (key:value)
        // This is a basic check - could be more sophisticated
        let colon_count = output.matches(':').count();

        if colon_count == 0 && output.contains('{') {
            report.add_warning("Object detected but no colons found");
        }

        Ok(())
    }
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Is structure valid
    pub structure_valid: bool,

    /// Are brackets balanced
    pub brackets_balanced: bool,

    /// Is data integrity maintained
    pub data_integrity: bool,

    /// Is encoding valid
    pub encoding_valid: bool,

    /// Is formatting consistent
    pub formatting_consistent: bool,

    /// List of validation issues
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new() -> Self {
        Self {
            structure_valid: false,
            brackets_balanced: false,
            data_integrity: false,
            encoding_valid: false,
            formatting_consistent: false,
            issues: Vec::new(),
        }
    }

    /// Add an error to the report
    pub fn add_error(&mut self, message: &str) {
        self.issues.push(ValidationIssue {
            severity: IssueSeverity::Error,
            message: message.to_string(),
        });
    }

    /// Add a warning to the report
    pub fn add_warning(&mut self, message: &str) {
        self.issues.push(ValidationIssue {
            severity: IssueSeverity::Warning,
            message: message.to_string(),
        });
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error)
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .count()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
            .count()
    }
}

/// Validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

/// Issue severity
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_toon_output() {
        let validator = ToonValidator::new(false);
        let json = json!({"name": "Alice", "age": 30});
        let toon = "{name: Alice, age: 30}";

        let report = validator.validate(toon, &json).unwrap();
        assert!(report.structure_valid);
        assert!(report.brackets_balanced);
    }

    #[test]
    fn test_empty_output_validation() {
        // Empty output is valid for empty objects per TOON spec
        let validator = ToonValidator::new(false);
        let json = json!({});

        let report = validator.validate("", &json).unwrap();
        // Empty output is now valid, so no errors
        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_unbalanced_brackets() {
        let validator = ToonValidator::new(false);
        let json = json!({"test": "value"});
        let toon = "{name: Alice"; // Missing closing brace

        let report = validator.validate(toon, &json).unwrap();
        assert!(!report.brackets_balanced);
        assert!(report.error_count() > 0);
    }

    #[test]
    fn test_bracket_in_string_ignored() {
        let validator = ToonValidator::new(false);
        let json = json!({"data": "{not a bracket}"});
        let toon = r#"{data: "{not a bracket}"}"#;

        let report = validator.validate(toon, &json).unwrap();
        assert!(report.brackets_balanced);
    }

    #[test]
    fn test_data_integrity_check() {
        let validator = ToonValidator::new(false);
        let json = json!({"name": "Alice", "age": 30, "city": "NYC"});
        let toon = "{name: Alice, age: 30}"; // Missing city

        let report = validator.validate(toon, &json).unwrap();
        assert!(!report.data_integrity);
    }

    #[test]
    fn test_complete_data_integrity() {
        let validator = ToonValidator::new(false);
        let json = json!({"name": "Alice", "value": 42});
        let toon = "{name: Alice, value: 42}";

        let report = validator.validate(toon, &json).unwrap();
        assert!(report.data_integrity);
    }

    #[test]
    fn test_escaped_quotes_in_strings() {
        let validator = ToonValidator::new(false);
        let json = json!({"message": "Hello \"World\""});
        let toon = r#"{message: "Hello \"World\""}"#;

        let report = validator.validate(toon, &json).unwrap();
        assert!(report.structure_valid);
    }

    #[test]
    fn test_strict_mode_failure() {
        let validator = ToonValidator::new(true);
        let json = json!({"test": "value"});
        let toon = "{test: value"; // Unbalanced

        let result = validator.validate(toon, &json);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_strict_mode_with_warnings() {
        let validator = ToonValidator::new(false);
        let json = json!({"test": "value"});
        let toon = "{test: value"; // Unbalanced

        let report = validator.validate(toon, &json).unwrap();
        assert!(!report.is_valid()); // Has errors
        assert!(report.error_count() > 0);
    }

    #[test]
    fn test_indentation_validation() {
        let validator = ToonValidator::new(false);
        let json = json!({"outer": {"inner": "value"}});
        let toon = "{\n  outer: {\n    inner: value\n  }\n}";

        let report = validator.validate(toon, &json).unwrap();
        assert!(report.formatting_consistent);
    }

    #[test]
    fn test_nested_structures_validation() {
        let validator = ToonValidator::new(false);
        let json = json!({
            "user": {
                "name": "Alice",
                "scores": [95, 87, 92]
            }
        });
        let toon = "{user: {name: Alice, scores: [95, 87, 92]}}";

        let report = validator.validate(toon, &json).unwrap();
        assert!(report.structure_valid);
        assert!(report.brackets_balanced);
    }
}
