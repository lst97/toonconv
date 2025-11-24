//! TOON (Token-Oriented Object Notation) Converter
//!
//! A Rust CLI tool for converting JSON data to TOON format with support for
//! various input sources and formatting options.

// Allow dead code for library exports that may not be used by the binary yet
#![allow(dead_code)]

pub mod cli;
pub mod conversion;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod validation;

// Re-export commonly used types
pub use conversion::{convert_json_to_toon, ConversionConfig, ConversionResult, ToonData};
pub use error::{ConversionError, ConversionErrorKind, ParseError};
pub use formatter::ToonFormatter;
pub use parser::JsonSource;

/// Convert JSON data to TOON format with default configuration
pub fn convert_json(json: &serde_json::Value) -> Result<String, ConversionError> {
    let config = ConversionConfig::default();
    convert_json_with_config(json, &config)
}

/// Convert JSON data to TOON format with custom configuration
pub fn convert_json_with_config(
    json: &serde_json::Value,
    config: &ConversionConfig,
) -> Result<String, ConversionError> {
    let result = convert_json_to_toon(json, config)?;
    Ok(result.content)
}
