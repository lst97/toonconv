//! JSON parsing and validation module

pub mod directory;
pub mod filter;
pub mod recursive;
pub mod validation;

use crate::error::{ParseError, ParseResult};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;

/// Types of JSON input sources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsonSourceType {
    /// Raw JSON string input
    String(String),
    /// Single JSON file path
    File(PathBuf),
    /// Directory containing multiple JSON files
    Directory(PathBuf),
    /// Standard input stream
    Stdin,
}

impl JsonSourceType {
    /// Create from string input
    pub fn from_string(s: String) -> Self {
        Self::String(s)
    }

    /// Create from file path
    pub fn from_file(path: PathBuf) -> Self {
        Self::File(path)
    }

    /// Create from directory path
    pub fn from_directory(path: PathBuf) -> Self {
        Self::Directory(path)
    }

    /// Create from stdin
    pub fn from_stdin() -> Self {
        Self::Stdin
    }

    /// Get a human-readable description of the source
    pub fn description(&self) -> String {
        match self {
            JsonSourceType::String(_) => "string input".to_string(),
            JsonSourceType::File(path) => format!("file: {}", path.display()),
            JsonSourceType::Directory(path) => format!("directory: {}", path.display()),
            JsonSourceType::Stdin => "standard input".to_string(),
        }
    }

    /// Check if the source exists and is accessible
    pub fn exists(&self) -> bool {
        match self {
            JsonSourceType::String(_) => true,
            JsonSourceType::File(path) => path.exists() && path.is_file(),
            JsonSourceType::Directory(path) => path.exists() && path.is_dir(),
            JsonSourceType::Stdin => true, // Assume stdin is always available
        }
    }

    /// Get the estimated size of the source in bytes (if known)
    pub fn estimated_size(&self) -> Option<u64> {
        match self {
            JsonSourceType::String(s) => Some(s.len() as u64),
            JsonSourceType::File(path) => {
                if let Ok(metadata) = std::fs::metadata(path) {
                    Some(metadata.len())
                } else {
                    None
                }
            }
            JsonSourceType::Directory(_) => None, // Don't estimate directory sizes
            JsonSourceType::Stdin => None,        // Unknown until read
        }
    }

    /// Check if this source represents a single JSON value (vs multiple files)
    pub fn is_single_value(&self) -> bool {
        matches!(
            self,
            JsonSourceType::String(_) | JsonSourceType::File(_) | JsonSourceType::Stdin
        )
    }
}

/// Source for parsing operations
#[derive(Debug, Clone)]
pub enum JsonSource {
    String(String),
    File(PathBuf),
    Directory(PathBuf),
    Stdin,
}

impl JsonSource {
    /// Parse JSON from this source
    pub fn parse(&self) -> ParseResult<serde_json::Value> {
        match self {
            JsonSource::String(content) => parse_from_string(content),
            JsonSource::File(path) => parse_from_file(path),
            JsonSource::Stdin => parse_from_stdin(),
            JsonSource::Directory(_) => Err(ParseError::new(
                "Cannot parse directory as single JSON value".to_string(),
                None,
            )),
        }
    }

    /// Get the source type
    pub fn source_type(&self) -> JsonSourceType {
        match self {
            JsonSource::String(s) => JsonSourceType::String(s.clone()),
            JsonSource::File(p) => JsonSourceType::File(p.clone()),
            JsonSource::Directory(p) => JsonSourceType::Directory(p.clone()),
            JsonSource::Stdin => JsonSourceType::Stdin,
        }
    }

    /// Read content as string (if possible)
    pub fn read_content(&self) -> Result<String, std::io::Error> {
        match self {
            JsonSource::String(content) => Ok(content.clone()),
            JsonSource::File(path) => std::fs::read_to_string(path),
            JsonSource::Stdin => {
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                Ok(buffer)
            }
            JsonSource::Directory(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot read directory as content",
            )),
        }
    }
}

/// Parse JSON from a string
fn parse_from_string(content: &str) -> ParseResult<serde_json::Value> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(ParseError::new("Empty JSON string".to_string(), None));
    }

    serde_json::from_str(trimmed).map_err(|e| {
        ParseError::new(
            format!("Invalid JSON: {}", e),
            extract_error_location(&e, trimmed),
        )
        .with_preview(get_error_preview(trimmed, &e))
    })
}

/// Parse JSON from a file
fn parse_from_file(path: &PathBuf) -> ParseResult<serde_json::Value> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ParseError::new(format!("Failed to read file: {}", e), None))?;

    parse_from_string(&content)
}

/// Parse JSON from standard input
fn parse_from_stdin() -> ParseResult<serde_json::Value> {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| ParseError::new(format!("Failed to read stdin: {}", e), None))?;

    parse_from_string(buffer.trim())
}

/// Extract error location from serde_json error
fn extract_error_location(error: &serde_json::Error, content: &str) -> Option<(usize, usize)> {
    // Try to extract line and column from error message
    let error_msg = error.to_string();

    // Look for patterns like "line X column Y"
    if let Some(line_start) = error_msg.find("line ") {
        if let Some(line_end) = error_msg.find(" column ") {
            let col_start = line_end;
            if let Some(col_end) = error_msg.find(" of ") {
                let line_str = &error_msg[line_start + 5..line_end];
                let col_str = &error_msg[col_start + 8..col_end];

                if let (Ok(line), Ok(col)) = (line_str.parse::<usize>(), col_str.parse::<usize>()) {
                    return Some((line, col));
                }
            }
        }
    }

    // Fallback: try to find position in content
    if let Some(pos) = error_msg.find("position ") {
        if let Some(end_pos) = error_msg.find(')') {
            let pos_str = &error_msg[pos + 9..end_pos];
            if let Ok(position) = pos_str.parse::<usize>() {
                let line = content[..position].chars().filter(|&c| c == '\n').count() + 1;
                let col = position - content[..position].rfind('\n').map_or(0, |p| p + 1);
                return Some((line, col + 1));
            }
        }
    }

    None
}

/// Get a preview of the error location
fn get_error_preview(content: &str, error: &serde_json::Error) -> String {
    if let Some((line, col)) = extract_error_location(error, content) {
        let lines: Vec<&str> = content.lines().collect();
        if line > 0 && line <= lines.len() {
            let error_line = lines[line - 1];
            let start = col.saturating_sub(1).min(error_line.len());
            let end = (col - 1 + 1).min(error_line.len());

            if start < end {
                return format!(
                    "...{}\n{}^",
                    &error_line[..start.max(error_line.len())],
                    " ".repeat(end - start)
                );
            }
        }
    }

    "Context not available".to_string()
}

/// Metadata about parsed JSON
#[derive(Debug, Clone)]
pub struct JsonMetadata {
    pub source_type: JsonSourceType,
    pub size_bytes: u64,
    pub line_count: usize,
    pub estimated_token_count: Option<usize>,
}

impl JsonMetadata {
    /// Create metadata for a string source
    pub fn from_string(source: &str, source_type: JsonSourceType) -> Self {
        let line_count = source.lines().count();
        let size_bytes = source.len() as u64;

        Self {
            source_type,
            size_bytes,
            line_count,
            estimated_token_count: Some(estimate_token_count(source)),
        }
    }

    /// Create metadata for a file source
    pub fn from_file(path: &PathBuf, content: &str) -> Result<Self, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        let source_type = JsonSourceType::File(path.clone());
        let line_count = content.lines().count();
        let size_bytes = metadata.len();

        Ok(Self {
            source_type,
            size_bytes,
            line_count,
            estimated_token_count: Some(estimate_token_count(content)),
        })
    }
}

/// Estimate token count for JSON content (rough approximation)
fn estimate_token_count(content: &str) -> usize {
    // Improved rough estimate: count structural elements + tokens
    let mut structural = 0usize;
    let mut whitespace = 0usize;
    let mut quote_count = 0usize;
    let mut token_count = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut last_was_token = false;

    for c in content.chars() {
        if escaped {
            escaped = false;
            last_was_token = true;
            continue;
        }

        if c == '\\' {
            escaped = true;
            continue;
        }

        if c == '"' && !escaped {
            quote_count += 1;
            in_string = !in_string;
            last_was_token = false;
            continue;
        }

        if c.is_whitespace() {
            whitespace += 1;
            last_was_token = false;
            continue;
        }

        if !in_string {
            match c {
                '{' | '}' | '[' | ']' | ',' | ':' => {
                    structural += 1;
                    last_was_token = false;
                }
                _ => {
                    if c.is_alphanumeric() && !last_was_token {
                        token_count += 1;
                        last_was_token = true;
                    } else if !c.is_alphanumeric() {
                        last_was_token = false;
                    }
                }
            }
        } else {
            // inside strings we consider words as tokens
            if c.is_alphanumeric() && !last_was_token {
                token_count += 1;
                last_was_token = true;
            } else if !c.is_alphanumeric() {
                last_was_token = false;
            }
        }
    }

    structural + whitespace + quote_count + token_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_source_type() {
        let source = JsonSourceType::String("{}".to_string());
        assert!(source.exists());
        assert_eq!(source.description(), "string input");

        let temp_file = NamedTempFile::new().unwrap();
        let file_source = JsonSourceType::File(temp_file.path().to_path_buf());
        assert!(file_source.exists());
    }

    #[test]
    fn test_parse_valid_json() {
        let json_str = r#"{"name": "test", "value": 42}"#;
        let source = JsonSource::String(json_str.to_string());
        let result = source.parse();
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.is_object());
    }

    #[test]
    fn test_parse_file_valid_json() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "{{\"name\": \"file\", \"value\": 123}}").unwrap();

        let source = JsonSource::File(tmp.path().to_path_buf());
        let result = source.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_json() {
        let json_str = r#"{"name": "test", "value": }"#;
        let source = JsonSource::String(json_str.to_string());
        let result = source.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_string() {
        let source = JsonSource::String("".to_string());
        let result = source.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_estimate_token_count() {
        let simple = r#"{"a": 1}"#;
        assert_eq!(estimate_token_count(simple), 8); // Rough estimate

        let complex = r#"
        {
            "users": [
                {"name": "Alice", "id": 1},
                {"name": "Bob", "id": 2}
            ]
        }
        "#;
        let count = estimate_token_count(complex);
        assert!(count > 20); // Should be significantly higher
    }
}
