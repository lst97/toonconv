//! Error types and handling infrastructure for JSON to TOON conversion

use anyhow::Error;
use std::fmt;
use std::path::PathBuf;

/// Core error types for the conversion process
#[derive(Debug, thiserror::Error)]
pub enum ConversionErrorKind {
    #[error("JSON parse error: {message}")]
    JsonParse {
        message: String,
        location: Option<(usize, usize)>,
    },

    #[error("TOON formatting error: {message}")]
    Formatting { message: String },

    #[error("IO error: {message}")]
    Io {
        message: String,
        path: Option<PathBuf>,
    },

    #[error("JSON too large: {size} bytes (limit: {limit} bytes)")]
    JsonTooLarge { size: usize, limit: usize },

    #[error("Memory limit exceeded: {size} bytes (limit: {limit} bytes)")]
    MemoryLimitExceeded { size: usize, limit: usize },

    #[error("Timeout exceeded: {timeout}s")]
    TimeoutExceeded { timeout: u64 },

    #[error("Invalid configuration: {message}")]
    Configuration { message: String },

    #[error("Circular reference detected")]
    CircularReference,

    #[error("Unsupported encoding: {encoding}")]
    UnsupportedEncoding { encoding: String },

    #[error("Conversion failed: {message}")]
    ConversionFailed { message: String },
}

impl ConversionErrorKind {
    pub fn json_parse(message: String, location: Option<(usize, usize)>) -> Self {
        Self::JsonParse { message, location }
    }

    pub fn formatting(message: String) -> Self {
        Self::Formatting { message }
    }

    pub fn io(message: String, path: Option<PathBuf>) -> Self {
        Self::Io { message, path }
    }

    pub fn configuration(message: String) -> Self {
        Self::Configuration { message }
    }
}

/// Main error type for conversion operations
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error(transparent)]
    ParseError(#[from] ParseError),

    #[error(transparent)]
    FormattingError(#[from] FormattingError),

    #[error("{kind}")]
    Conversion {
        kind: ConversionErrorKind,
        source: Option<anyhow::Error>,
    },

    #[error(transparent)]
    Other(#[from] Error),
}

impl ConversionError {
    pub fn parse(message: String, location: Option<(usize, usize)>) -> Self {
        Self::ParseError(ParseError::new(message, location))
    }

    pub fn formatting(message: String) -> Self {
        Self::FormattingError(FormattingError::invalid_structure(message))
    }

    pub fn conversion(kind: ConversionErrorKind) -> Self {
        Self::Conversion { kind, source: None }
    }

    pub fn conversion_with_source(kind: ConversionErrorKind, source: anyhow::Error) -> Self {
        Self::Conversion {
            kind,
            source: Some(source),
        }
    }

    pub fn other(error: Error) -> Self {
        Self::Other(error)
    }

    /// Create a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::ParseError(err) => {
                if let Some((line, col)) = err.location {
                    format!(
                        "JSON parse error at line {}, column {}: {}",
                        line, col, err.message
                    )
                } else {
                    format!("JSON parse error: {}", err.message)
                }
            }
            Self::FormattingError(err) => {
                format!("TOON formatting error: {}", err)
            }
            Self::Conversion { kind, .. } => match kind {
                ConversionErrorKind::JsonTooLarge { size, limit } => {
                    format!(
                        "JSON file too large: {} bytes (limit: {} bytes)",
                        size, limit
                    )
                }
                ConversionErrorKind::MemoryLimitExceeded { size, limit } => {
                    format!(
                        "Memory limit exceeded: {} bytes (limit: {} bytes)",
                        size, limit
                    )
                }
                ConversionErrorKind::TimeoutExceeded { timeout } => {
                    format!("Conversion timeout: {} seconds", timeout)
                }
                ConversionErrorKind::UnsupportedEncoding { encoding } => {
                    format!("Unsupported encoding: {}", encoding)
                }
                ConversionErrorKind::CircularReference => {
                    "Circular reference detected in JSON data".to_string()
                }
                _ => self.to_string(),
            },
            Self::Other(err) => {
                format!("Unexpected error: {}", err)
            }
        }
    }
}

/// JSON parsing errors
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub location: Option<(usize, usize)>,
    pub input_preview: Option<String>,
}

impl ParseError {
    pub fn new(message: String, location: Option<(usize, usize)>) -> Self {
        Self {
            message,
            location,
            input_preview: None,
        }
    }

    pub fn with_preview(mut self, preview: String) -> Self {
        self.input_preview = Some(preview);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some((line, col)) = self.location {
            write!(f, " at line {}, column {}", line, col)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// TOON formatting errors
#[derive(Debug, thiserror::Error)]
pub enum FormattingError {
    #[error("Invalid TOON structure: {message}")]
    InvalidStructure { message: String },

    #[error("Quoting error: {message}")]
    QuotingError { message: String },

    #[error("Indentation error: {message}")]
    IndentationError { message: String },

    #[error("Schema error: {message}")]
    SchemaError { message: String },
}

impl FormattingError {
    pub fn invalid_structure(message: String) -> Self {
        Self::InvalidStructure { message }
    }

    pub fn quoting(message: String) -> Self {
        Self::QuotingError { message }
    }

    pub fn indentation(message: String) -> Self {
        Self::IndentationError { message }
    }

    pub fn schema(message: String) -> Self {
        Self::SchemaError { message }
    }
}

/// Error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub source_path: Option<PathBuf>,
    pub line_number: Option<usize>,
    pub column_number: Option<usize>,
    pub additional_info: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            source_path: None,
            line_number: None,
            column_number: None,
            additional_info: None,
        }
    }

    pub fn with_source_path(mut self, path: PathBuf) -> Self {
        self.source_path = Some(path);
        self
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line_number = Some(line);
        self.column_number = Some(column);
        self
    }

    pub fn with_additional_info(mut self, info: String) -> Self {
        self.additional_info = Some(info);
        self
    }
}

/// Result type for conversion operations
pub type ConversionResult<T> = Result<T, ConversionError>;

/// Convenience result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Convenience result type for formatting operations
pub type FormattingResult<T> = Result<T, FormattingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::new("Unexpected token".to_string(), Some((5, 10)));
        assert_eq!(error.to_string(), "Unexpected token at line 5, column 10");
    }

    #[test]
    fn test_conversion_error_user_message() {
        let error = ConversionError::parse("Invalid JSON".to_string(), Some((1, 5)));
        assert!(error
            .user_message()
            .contains("JSON parse error at line 1, column 5"));
    }

    #[test]
    fn test_conversion_error_kind_variants() {
        let kinds = vec![
            ConversionErrorKind::json_parse("test".to_string(), None),
            ConversionErrorKind::formatting("test".to_string()),
            ConversionErrorKind::configuration("test".to_string()),
        ];

        for kind in kinds {
            let error = ConversionError::conversion(kind);
            assert!(!error.user_message().is_empty());
        }
    }
}
