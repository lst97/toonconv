//! Smart string quoting logic per TOON specification
//!
//! TOON uses smart quoting rules to minimize token usage while
//! preserving data integrity. Strings are quoted only when necessary.

use crate::error::FormattingResult;

/// Smart quoting engine for TOON strings
pub struct QuoteEngine {
    delimiter: String,
}

impl QuoteEngine {
    /// Create a new quote engine with the specified delimiter
    pub fn new(delimiter: String) -> Self {
        Self { delimiter }
    }

    /// Determine if a string needs quoting according to TOON rules
    ///
    /// Quoting Rules (in priority order):
    /// 1. Quote if contains TOON control/delimiter characters: : , \n { } [ ] or configured delimiter
    /// 2. Quote if begins or ends with whitespace
    /// 3. Quote if empty string
    /// 4. Quote if equals literal values: "true", "false", "null"
    /// 5. Quote if looks like a number
    /// 6. Otherwise, no quotes needed
    pub fn needs_quoting(&self, value: &str) -> bool {
        // Rule 3: Empty string
        if value.is_empty() {
            return true;
        }

        // Rule 4: Literal values that need preservation as strings
        if value == "true" || value == "false" || value == "null" {
            return true;
        }

        // Rule 5: Numeric strings (would be interpreted as numbers)
        if self.looks_like_number(value) {
            return true;
        }

        // Rule 2: Leading or trailing whitespace
        if let Some(first_char) = value.chars().next() {
            if first_char.is_whitespace() {
                return true;
            }
        }
        if let Some(last_char) = value.chars().last() {
            if last_char.is_whitespace() {
                return true;
            }
        }

        // Rule 1: TOON control and delimiter characters
        if self.contains_control_characters(value) {
            return true;
        }

        // No quoting needed
        false
    }

    /// Quote a string according to TOON escaping rules
    pub fn quote(&self, value: &str) -> FormattingResult<String> {
        if value.is_empty() {
            return Ok("\"\"".to_string());
        }

        let mut result = String::with_capacity(value.len() + 2);
        result.push('"');

        for ch in value.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\x08' => result.push_str("\\b"),
                '\x0C' => result.push_str("\\f"),
                _ if ch.is_control() => {
                    // Unicode escape for other control characters
                    result.push_str(&format!("\\u{:04x}", ch as u32));
                }
                _ => result.push(ch),
            }
        }

        result.push('"');
        Ok(result)
    }

    /// Format a string with smart quoting
    pub fn format(&self, value: &str) -> FormattingResult<String> {
        if self.needs_quoting(value) {
            self.quote(value)
        } else {
            Ok(value.to_string())
        }
    }

    /// Check if string contains TOON control characters
    fn contains_control_characters(&self, value: &str) -> bool {
        // TOON structural characters
        const TOON_CONTROL_CHARS: &[char] = &[':', ',', '\n', '\r', '{', '}', '[', ']'];

        for ch in value.chars() {
            // Check TOON control characters
            if TOON_CONTROL_CHARS.contains(&ch) {
                return true;
            }

            // Check configured delimiter
            if self.delimiter.contains(ch) {
                return true;
            }

            // Check other control characters
            if ch.is_control() {
                return true;
            }
        }

        false
    }

    /// Check if string looks like a number
    fn looks_like_number(&self, value: &str) -> bool {
        // Try to parse as f64
        if value.parse::<f64>().is_ok() {
            return true;
        }

        // Check for special number formats
        if value == "Infinity" || value == "-Infinity" || value == "NaN" {
            return true;
        }

        // Check for scientific notation pattern (e.g., "1e10", "2.5E-3")
        // Must have digits before 'e'/'E' and after (with optional sign)
        if let Some(e_pos) = value.find('e').or_else(|| value.find('E')) {
            // Check if there are digits before 'e'
            let before = &value[..e_pos];
            if before.is_empty() || !before.chars().any(|c| c.is_ascii_digit()) {
                return false;
            }

            // Check if there's a valid exponent after 'e'
            let after = &value[e_pos + 1..];
            if after.is_empty() {
                return false;
            }

            // After 'e' should be: optional sign followed by digits
            let exponent = if after.starts_with('+') || after.starts_with('-') {
                &after[1..]
            } else {
                after
            };

            return !exponent.is_empty() && exponent.chars().all(|c| c.is_ascii_digit());
        }

        false
    }
}

/// Convenience function to check if quoting is needed
pub fn needs_quoting(value: &str, delimiter: &str) -> bool {
    let engine = QuoteEngine::new(delimiter.to_string());
    engine.needs_quoting(value)
}

/// Convenience function to quote a string
pub fn quote_string(value: &str) -> FormattingResult<String> {
    let engine = QuoteEngine::new(",".to_string());
    engine.quote(value)
}

/// Convenience function to format with smart quoting
pub fn smart_quote(value: &str, delimiter: &str) -> FormattingResult<String> {
    let engine = QuoteEngine::new(delimiter.to_string());
    engine.format(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string_needs_quoting() {
        let engine = QuoteEngine::new(",".to_string());
        assert!(engine.needs_quoting(""));
    }

    #[test]
    fn test_literal_values_need_quoting() {
        let engine = QuoteEngine::new(",".to_string());
        assert!(engine.needs_quoting("true"));
        assert!(engine.needs_quoting("false"));
        assert!(engine.needs_quoting("null"));
    }

    #[test]
    fn test_numeric_strings_need_quoting() {
        let engine = QuoteEngine::new(",".to_string());
        assert!(engine.needs_quoting("42"));
        assert!(engine.needs_quoting("3.14"));
        assert!(engine.needs_quoting("-100"));
        assert!(engine.needs_quoting("1e10"));
        assert!(engine.needs_quoting("Infinity"));
        assert!(engine.needs_quoting("NaN"));
    }

    #[test]
    fn test_whitespace_needs_quoting() {
        let engine = QuoteEngine::new(",".to_string());
        assert!(engine.needs_quoting(" hello"));
        assert!(engine.needs_quoting("hello "));
        assert!(engine.needs_quoting(" hello "));
        assert!(engine.needs_quoting("\thello"));
    }

    #[test]
    fn test_control_characters_need_quoting() {
        let engine = QuoteEngine::new(",".to_string());
        assert!(engine.needs_quoting("hello:world"));
        assert!(engine.needs_quoting("a,b,c"));
        assert!(engine.needs_quoting("line1\nline2"));
        assert!(engine.needs_quoting("{data}"));
        assert!(engine.needs_quoting("[array]"));
    }

    #[test]
    fn test_delimiter_needs_quoting() {
        let engine = QuoteEngine::new("\t".to_string());
        assert!(engine.needs_quoting("hello\tworld"));

        let pipe_engine = QuoteEngine::new("|".to_string());
        assert!(pipe_engine.needs_quoting("a|b"));
    }

    #[test]
    fn test_simple_strings_no_quoting() {
        let engine = QuoteEngine::new(",".to_string());
        assert!(!engine.needs_quoting("hello"));
        assert!(!engine.needs_quoting("world"));
        assert!(!engine.needs_quoting("Alice"));
        assert!(!engine.needs_quoting("test123"));
        assert!(!engine.needs_quoting("hello world")); // Internal space is OK
    }

    #[test]
    fn test_quote_empty_string() {
        let engine = QuoteEngine::new(",".to_string());
        assert_eq!(engine.quote("").unwrap(), "\"\"");
    }

    #[test]
    fn test_quote_escape_sequences() {
        let engine = QuoteEngine::new(",".to_string());
        assert_eq!(engine.quote("hello\"world").unwrap(), "\"hello\\\"world\"");
        assert_eq!(engine.quote("path\\file").unwrap(), "\"path\\\\file\"");
        assert_eq!(engine.quote("line1\nline2").unwrap(), "\"line1\\nline2\"");
        assert_eq!(engine.quote("tab\there").unwrap(), "\"tab\\there\"");
        assert_eq!(
            engine.quote("carriage\rreturn").unwrap(),
            "\"carriage\\rreturn\""
        );
    }

    #[test]
    fn test_format_with_smart_quoting() {
        let engine = QuoteEngine::new(",".to_string());

        // Should quote
        assert_eq!(engine.format("true").unwrap(), "\"true\"");
        assert_eq!(engine.format("42").unwrap(), "\"42\"");
        assert_eq!(engine.format(" hello").unwrap(), "\" hello\"");
        assert_eq!(engine.format("").unwrap(), "\"\"");

        // Should not quote
        assert_eq!(engine.format("hello").unwrap(), "hello");
        assert_eq!(engine.format("Alice").unwrap(), "Alice");
        assert_eq!(engine.format("test data").unwrap(), "test data");
    }

    #[test]
    fn test_control_character_escaping() {
        let engine = QuoteEngine::new(",".to_string());

        // Backspace
        assert_eq!(engine.quote("\x08").unwrap(), "\"\\b\"");

        // Form feed
        assert_eq!(engine.quote("\x0C").unwrap(), "\"\\f\"");

        // Other control characters use unicode escape
        assert!(engine.quote("\x01").unwrap().contains("\\u"));
    }

    #[test]
    fn test_convenience_functions() {
        assert!(needs_quoting("true", ","));
        assert!(!needs_quoting("hello", ","));

        assert_eq!(quote_string("hello\"world").unwrap(), "\"hello\\\"world\"");

        assert_eq!(smart_quote("true", ",").unwrap(), "\"true\"");
        assert_eq!(smart_quote("hello", ",").unwrap(), "hello");
    }
}
