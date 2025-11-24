//! Configuration options for JSON to TOON conversion

use std::time::Duration;

/// Array delimiter options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DelimiterType {
    /// Comma delimiter (,)
    Comma,
    /// Tab delimiter (\\t)
    Tab,
    /// Pipe delimiter (|)
    Pipe,
}

impl DelimiterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DelimiterType::Comma => ",",
            DelimiterType::Tab => "\t",
            DelimiterType::Pipe => "|",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "comma" | "," => Ok(DelimiterType::Comma),
            "tab" | "\t" => Ok(DelimiterType::Tab),
            "pipe" | "|" => Ok(DelimiterType::Pipe),
            other => Err(format!(
                "Invalid delimiter '{}'. Use 'comma', 'tab', or 'pipe'",
                other
            )),
        }
    }
}

/// String quoting strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteStrategy {
    /// Smart quoting (only quote when necessary)
    Smart,
    /// Always quote all strings
    Always,
    /// Never quote strings
    Never,
}

impl QuoteStrategy {
    pub fn should_quote(&self, value: &str, delimiter: DelimiterType) -> bool {
        match self {
            QuoteStrategy::Always => true,
            QuoteStrategy::Never => false,
            QuoteStrategy::Smart => needs_quoting(value, delimiter),
        }
    }
}

/// Check if a string needs quoting according to TOON rules
fn needs_quoting(value: &str, delimiter: DelimiterType) -> bool {
    if value.is_empty() {
        return true;
    }

    // Keywords that need quoting
    if value == "null" || value == "true" || value == "false" {
        return true;
    }

    // Numeric strings need quoting
    if value.parse::<f64>().is_ok() {
        return true;
    }

    // Leading or trailing whitespace
    if value.starts_with(' ') || value.ends_with(' ') {
        return true;
    }

    // Contains structural characters
    let structural_chars = ":[]{}";
    if value.chars().any(|c| structural_chars.contains(c)) {
        return true;
    }

    // Contains control characters
    if value.chars().any(|c| c.is_control()) {
        return true;
    }

    // Contains current delimiter
    if value.contains(delimiter.as_str()) {
        return true;
    }

    false
}

/// Conversion configuration options
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Spaces per indentation level (0-8)
    pub indent_size: u8,
    /// Array delimiter
    pub delimiter: DelimiterType,
    /// Include length markers in arrays
    pub length_marker: bool,
    /// String quoting strategy
    pub quote_strings: QuoteStrategy,
    /// Maximum memory usage limit in bytes
    pub memory_limit: usize,
    /// Maximum processing timeout
    pub timeout: Duration,
    /// Enable SIMD performance optimizations
    pub enable_simd: bool,
    /// Pretty-print output (vs compact)
    pub pretty: bool,
    /// Validate TOON output after conversion
    pub validate_output: bool,
    /// Include schema declarations for arrays
    pub include_schema: bool,
    /// Maximum nesting depth
    pub max_depth: Option<usize>,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            indent_size: 2,
            delimiter: DelimiterType::Comma,
            length_marker: true,
            quote_strings: QuoteStrategy::Smart,
            memory_limit: 100 * 1024 * 1024,   // 100MB
            timeout: Duration::from_secs(300), // 5 minutes
            enable_simd: false,
            pretty: true,
            validate_output: true,
            include_schema: true,
            max_depth: Some(1000), // Reasonable limit to prevent stack overflow
        }
    }
}

impl ConversionConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration optimized for small files (<1MB)
    pub fn small_files() -> Self {
        Self {
            memory_limit: 10 * 1024 * 1024,   // 10MB
            timeout: Duration::from_secs(30), // 30 seconds
            ..Default::default()
        }
    }

    /// Create configuration optimized for large files (>100MB)
    pub fn large_files() -> Self {
        Self {
            memory_limit: 1024 * 1024 * 1024,   // 1GB
            timeout: Duration::from_secs(1800), // 30 minutes
            enable_simd: true,
            validate_output: false, // Skip validation for performance
            ..Default::default()
        }
    }

    /// Create configuration for batch processing
    pub fn batch_processing() -> Self {
        Self {
            memory_limit: 512 * 1024 * 1024,   // 512MB
            timeout: Duration::from_secs(600), // 10 minutes
            enable_simd: true,
            pretty: false, // Compact output for batch processing
            validate_output: false,
            ..Default::default()
        }
    }

    /// Set indentation size
    pub fn with_indent_size(mut self, size: u8) -> Result<Self, String> {
        if size > 8 {
            return Err("Indent size must be 0-8 spaces".to_string());
        }
        self.indent_size = size;
        Ok(self)
    }

    /// Set array delimiter
    pub fn with_delimiter(mut self, delimiter: DelimiterType) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Enable length markers in arrays
    pub fn with_length_marker(mut self, enabled: bool) -> Self {
        self.length_marker = enabled;
        self
    }

    /// Set string quoting strategy
    pub fn with_quote_strategy(mut self, strategy: QuoteStrategy) -> Self {
        self.quote_strings = strategy;
        self
    }

    /// Set memory limit
    pub fn with_memory_limit(mut self, limit_bytes: usize) -> Self {
        self.memory_limit = limit_bytes;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable SIMD optimizations
    pub fn with_simd(mut self, enabled: bool) -> Self {
        self.enable_simd = enabled;
        self
    }

    /// Enable/disable pretty printing
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Enable/disable output validation
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_output = validate;
        self
    }

    /// Set maximum nesting depth
    pub fn with_max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    /// Validate configuration consistency
    pub fn validate(&self) -> Result<(), String> {
        // Check indent size bounds
        if self.indent_size > 8 {
            return Err("Indent size must be 0-8 spaces".to_string());
        }

        // Check memory limit bounds
        if self.memory_limit < 1024 {
            return Err("Memory limit must be at least 1KB".to_string());
        }

        // Check timeout bounds
        if self.timeout.as_secs() == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        // Check max depth bounds
        if let Some(depth) = self.max_depth {
            if depth == 0 {
                return Err("Max depth must be at least 1".to_string());
            }
        }

        Ok(())
    }

    /// Get the appropriate JSON parser based on configuration
    pub fn json_parser_type(&self) -> JsonParserType {
        if self.enable_simd {
            JsonParserType::SimdJson
        } else {
            JsonParserType::SerdeJson
        }
    }
}

/// Types of JSON parsers available
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsonParserType {
    /// Standard serde_json parser
    SerdeJson,
    /// SIMD-optimized parser
    SimdJson,
}

/// Validation strategy for TOON output
#[derive(Debug, Clone)]
pub enum ValidationStrategy {
    /// No validation
    None,
    /// Basic syntax validation
    Basic,
    /// Full compliance validation
    Full,
}

/// Performance profile for different use cases
#[derive(Debug, Clone)]
pub enum PerformanceProfile {
    /// Optimized for speed
    Speed,
    /// Optimized for memory usage
    Memory,
    /// Balanced approach
    Balanced,
    /// Custom configuration
    Custom(ConversionConfig),
}

impl PerformanceProfile {
    pub fn to_config(&self) -> ConversionConfig {
        match self {
            PerformanceProfile::Speed => ConversionConfig::large_files(),
            PerformanceProfile::Memory => ConversionConfig::small_files(),
            PerformanceProfile::Balanced => ConversionConfig::default(),
            PerformanceProfile::Custom(config) => config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ConversionConfig::default();
        assert_eq!(config.indent_size, 2);
        assert_eq!(config.delimiter, DelimiterType::Comma);
        assert!(config.length_marker);
        assert_eq!(config.quote_strings, QuoteStrategy::Smart);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ConversionConfig::default();
        assert!(config.validate().is_ok());

        config.indent_size = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_quote_strategy() {
        assert!(QuoteStrategy::Smart.should_quote("", DelimiterType::Comma));
        assert!(QuoteStrategy::Smart.should_quote("null", DelimiterType::Comma));
        assert!(!QuoteStrategy::Smart.should_quote("hello", DelimiterType::Comma));
        assert!(QuoteStrategy::Smart.should_quote("hello,world", DelimiterType::Comma));
    }

    #[test]
    fn test_delimiter_from_str() {
        assert_eq!(
            DelimiterType::from_str("comma").unwrap(),
            DelimiterType::Comma
        );
        assert_eq!(DelimiterType::from_str("tab").unwrap(), DelimiterType::Tab);
        assert_eq!(
            DelimiterType::from_str("pipe").unwrap(),
            DelimiterType::Pipe
        );
        assert!(DelimiterType::from_str("invalid").is_err());
    }

    #[test]
    fn test_performance_profiles() {
        let speed = PerformanceProfile::Speed;
        let memory = PerformanceProfile::Memory;
        let balanced = PerformanceProfile::Balanced;

        assert!(speed.to_config().enable_simd);
        assert!(memory.to_config().memory_limit < 100 * 1024 * 1024);
        assert!(balanced.to_config().validate_output);
    }
}
