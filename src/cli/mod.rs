//! Command-line interface module

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::time::Duration;

use crate::conversion::{ConversionConfig, ConversionResult};
use crate::conversion::config::{DelimiterType, QuoteStrategy};
use crate::error::{ConversionError, ConversionErrorKind};

pub mod path_mapping;

/// Main CLI arguments
#[derive(Parser, Debug, Clone)]
#[command(name = "toonconv")]
#[command(about = "Convert JSON to TOON (Token-Oriented Object Notation) format")]
#[command(version = "0.1.0")]
#[command(long_about = None)]
pub struct Args {
    /// Input JSON source (string, file, or directory)
    #[arg()]
    pub input: Option<String>,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Read JSON from standard input
    #[arg(long)]
    pub stdin: bool,

    /// Recursively process directories
    #[arg(long)]
    pub recursive: bool,

    /// Spaces per indentation level (0-8, default: 2)
    #[arg(long)]
    pub indent: Option<u8>,

    /// Array delimiter: comma, tab, or pipe (default: comma)
    #[arg(long)]
    pub delimiter: Option<Delimiter>,

    /// Include length markers in arrays
    #[arg(long)]
    pub length_marker: bool,

    /// Disable pretty-printing
    #[arg(long)]
    pub plain: bool,

    /// Maximum memory usage limit (e.g., 100MB, default: 100MB)
    #[arg(long)]
    pub memory_limit: Option<String>,

    /// Maximum processing time in seconds (default: 300)
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Use SIMD-optimized JSON parser
    #[arg(long)]
    pub simd: bool,

    /// Only validate JSON, don't convert
    #[arg(long)]
    pub validate_only: bool,

    /// Output conversion statistics
    #[arg(long)]
    pub stats: bool,

    /// Enable verbose logging
    #[arg(long)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(long)]
    pub quiet: bool,

    /// Continue converting other files when one file fails
    #[arg(long)]
    pub continue_on_error: bool,

    /// Subcommands for advanced operations
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// CLI subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Validate JSON files without conversion
    Validate {
        /// Input path (file or directory)
        input: String,
        /// Output validation report
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Generate example TOON files
    Examples {
        /// Output directory for examples
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Number of examples to generate
        #[arg(long)]
        count: Option<u32>,
    },
    /// Performance benchmarking
    Benchmark {
        /// Input file for benchmarking
        input: String,
        /// Number of iterations
        #[arg(long)]
        iterations: Option<u32>,
        /// Output benchmark results
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

/// Delimiter types for CLI
#[derive(ValueEnum, Debug, Clone)]
pub enum Delimiter {
    #[value(name = "comma", alias = ",")]
    Comma,
    #[value(name = "tab", alias = "\t")]
    Tab,
    #[value(name = "pipe", alias = "|")]
    Pipe,
}

impl From<Delimiter> for DelimiterType {
    fn from(delimiter: Delimiter) -> Self {
        match delimiter {
            Delimiter::Comma => DelimiterType::Comma,
            Delimiter::Tab => DelimiterType::Tab,
            Delimiter::Pipe => DelimiterType::Pipe,
        }
    }
}

/// CLI configuration
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub args: Args,
    pub conversion_config: ConversionConfig,
}

impl CliConfig {
    /// Create CLI configuration from arguments
    pub fn from_args(args: Args) -> ConversionResult<Self> {
        let conversion_config = Self::create_conversion_config(&args)?;

        Ok(Self {
            args,
            conversion_config,
        })
    }

    /// Check if we should continue on error
    pub fn continue_on_error(&self) -> bool {
        self.args.continue_on_error
    }

    /// Create conversion configuration from CLI arguments
    fn create_conversion_config(args: &Args) -> ConversionResult<ConversionConfig> {
        let delimiter = args.delimiter.as_ref().map(|d| d.clone().into()).unwrap_or(DelimiterType::Comma);
        let memory_limit = parse_memory_limit(&args.memory_limit)?;
        let timeout = Duration::from_secs(args.timeout.unwrap_or(300));

        let config = ConversionConfig {
            indent_size: args.indent.unwrap_or(2),
            delimiter,
            length_marker: args.length_marker,
            quote_strings: QuoteStrategy::Smart,
            memory_limit,
            timeout,
            enable_simd: args.simd,
            pretty: !args.plain,
            validate_output: true,
            include_schema: true,
            max_depth: Some(1000),
        };

        // Validate configuration
        config.validate()
            .map_err(|e| ConversionError::conversion(ConversionErrorKind::configuration(e)))?;

        Ok(config)
    }

    /// Check if quiet mode is enabled
    pub fn is_quiet(&self) -> bool {
        self.args.quiet
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.args.verbose
    }

    /// Check if stats output is requested
    pub fn want_stats(&self) -> bool {
        self.args.stats
    }

    /// Check if only validation is requested
    pub fn is_validate_only(&self) -> bool {
        self.args.validate_only
    }

    /// Get input source description
    pub fn input_description(&self) -> String {
        if self.args.stdin {
            "standard input".to_string()
        } else if let Some(input) = &self.args.input {
            format!("'{}'", input)
        } else {
            "no input specified".to_string()
        }
    }

    /// Get output destination description
    pub fn output_description(&self) -> String {
        if let Some(output) = &self.args.output {
            format!("'{}'", output.display())
        } else {
            "standard output".to_string()
        }
    }
}

/// Parse memory limit string (e.g., "100MB", "1GB", "500KB")
fn parse_memory_limit(limit: &Option<String>) -> ConversionResult<usize> {
    match limit {
        None => Ok(100 * 1024 * 1024), // 100MB default
        Some(limit_str) => {
            let limit_str = limit_str.trim().to_uppercase();
            
            if limit_str.ends_with("MB") {
                let size = &limit_str[..limit_str.len() - 2];
                let mb = size.parse::<f64>()
                    .map_err(|_| ConversionError::conversion(ConversionErrorKind::Configuration {
                        message: format!("Invalid memory limit: {}", limit_str)
                    }))?;
                Ok((mb * 1024.0 * 1024.0) as usize)
            } else if limit_str.ends_with("KB") {
                let size = &limit_str[..limit_str.len() - 2];
                let kb = size.parse::<f64>()
                    .map_err(|_| ConversionError::conversion(ConversionErrorKind::Configuration {
                        message: format!("Invalid memory limit: {}", limit_str)
                    }))?;
                Ok((kb * 1024.0) as usize)
            } else if limit_str.ends_with("GB") {
                let size = &limit_str[..limit_str.len() - 2];
                let gb = size.parse::<f64>()
                    .map_err(|_| ConversionError::conversion(ConversionErrorKind::Configuration {
                        message: format!("Invalid memory limit: {}", limit_str)
                    }))?;
                Ok((gb * 1024.0 * 1024.0 * 1024.0) as usize)
            } else if limit_str.ends_with("B") {
                let size = &limit_str[..limit_str.len() - 1];
                size.parse::<usize>()
                    .map_err(|_| ConversionError::conversion(ConversionErrorKind::Configuration {
                        message: format!("Invalid memory limit: {}", limit_str)
                    }))
            } else {
                // Assume bytes
                limit_str.parse::<usize>()
                    .map_err(|_| ConversionError::conversion(ConversionErrorKind::Configuration {
                        message: format!("Invalid memory limit: {}", limit_str)
                    }))
            }
        }
    }
}

/// CLI utilities and helpers
pub struct CliUtils;

impl CliUtils {
    /// Format a file size in human-readable format
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Format a duration in human-readable format
    pub fn format_duration(duration: Duration) -> String {
        let total_millis = duration.as_millis();

        if total_millis < 1000 {
            format!("{}ms", total_millis)
        } else if total_millis < 60_000 {
            format!("{:.1}s", total_millis as f64 / 1000.0)
        } else {
            let minutes = total_millis / 60_000;
            let seconds = (total_millis % 60_000) / 1000;
            format!("{}m {}s", minutes, seconds)
        }
    }

    /// Format a percentage
    pub fn format_percentage(value: f32) -> String {
        format!("{:.1}%", value)
    }

    /// Create a progress bar for file processing
    pub fn create_progress_bar(total: u64) -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::new(total);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    }

    /// Show a success message (if not in quiet mode)
    pub fn show_success(message: &str, quiet: bool) {
        if !quiet {
            println!("✓ {}", message);
        }
    }

    /// Show an error message
    pub fn show_error(message: &str) {
        eprintln!("✗ {}", message);
    }

    /// Show a warning message (if not in quiet mode)
    pub fn show_warning(message: &str, quiet: bool) {
        if !quiet {
            eprintln!("⚠ {}", message);
        }
    }

    /// Check if output should be colored
    pub fn should_use_color() -> bool {
        // Check if stdout is a terminal and supports color
        atty::is(atty::Stream::Stdout) && std::env::var("NO_COLOR").is_err()
    }

    /// Get the terminal size
    pub fn get_terminal_size() -> (u16, u16) {
        terminal_size::terminal_size()
            .map(|(width, height)| (width.0, height.0))
            .unwrap_or((80, 24))
    }
}

/// Handle CLI errors with user-friendly messages
pub fn handle_error(error: &ConversionError) {
    let message = error.user_message();
    CliUtils::show_error(&message);

    // Provide helpful suggestions
    if error.to_string().contains("JSON parse error") {
        eprintln!("\nTip: Use --validate-only to check JSON syntax before conversion");
    } else if error.to_string().contains("Memory limit exceeded") {
        eprintln!("\nTip: Use --memory-limit to increase memory allowance");
    } else if error.to_string().contains("timeout") {
        eprintln!("\nTip: Use --timeout to increase processing time limit");
    }

    // Show usage hint
    eprintln!("\nTry 'toonconv --help' for usage information.");
}

/// Command execution result
pub type CliResult<T> = Result<T, ConversionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_limit_parsing() {
        assert_eq!(parse_memory_limit(&Some("1MB".to_string())).unwrap(), 1024 * 1024);
        assert_eq!(parse_memory_limit(&Some("500KB".to_string())).unwrap(), 500 * 1024);
        assert_eq!(parse_memory_limit(&Some("2GB".to_string())).unwrap(), 2 * 1024 * 1024 * 1024);
        assert_eq!(parse_memory_limit(&Some("1024".to_string())).unwrap(), 1024);
    }

    #[test]
    fn test_cli_config_creation() {
        let args = Args {
            input: Some("test.json".to_string()),
            output: None,
            stdin: false,
            recursive: false,
            indent: Some(4),
            delimiter: Some(Delimiter::Tab),
            length_marker: true,
            plain: false,
            memory_limit: Some("50MB".to_string()),
            timeout: Some(600),
            simd: true,
            validate_only: false,
            stats: false,
            verbose: false,
            quiet: false,
            command: None,
            continue_on_error: false,
        };

        let config = CliConfig::from_args(args).unwrap();
        assert_eq!(config.conversion_config.indent_size, 4);
        assert_eq!(config.conversion_config.delimiter, DelimiterType::Tab);
        assert!(config.conversion_config.length_marker);
        assert!(config.conversion_config.enable_simd);
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(CliUtils::format_file_size(1024), "1.0 KB");
        assert_eq!(CliUtils::format_file_size(1048576), "1.0 MB");
        assert_eq!(CliUtils::format_file_size(512), "512 B");
    }

    #[test]
    fn test_duration_formatting() {
        let duration = Duration::from_millis(500);
        assert_eq!(CliUtils::format_duration(duration), "500ms");

        let duration = Duration::from_millis(1500);
        assert_eq!(CliUtils::format_duration(duration), "1.5s");

        let duration = Duration::from_secs(90);
        assert_eq!(CliUtils::format_duration(duration), "1m 30s");
    }
}
