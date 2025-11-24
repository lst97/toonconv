// Allow dead code for features exported but not yet used by the CLI
#![allow(dead_code)]

use clap::Parser;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;

mod cli;
mod conversion;
mod error;
mod formatter;
mod parser;
mod validation;

use crate::conversion::{convert_json_to_toon, ConversionConfig};
use crate::parser::JsonSource;

/// TOON (Token-Oriented Object Notation) Converter
#[derive(Parser, Debug)]
#[command(name = "toonconv")]
#[command(about = "Convert JSON to TOON (Token-Oriented Object Notation) format")]
#[command(version = "0.1.0")]
struct CliArgs {
    /// Input JSON source (string, file, or directory)
    #[arg()]
    input: Option<String>,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Read JSON from standard input
    #[arg(long)]
    stdin: bool,

    /// Recursively process directories
    #[arg(long)]
    recursive: bool,

    /// Spaces per indentation level (0-8, default: 2)
    #[arg(long)]
    indent: Option<u8>,

    /// Array delimiter: comma, tab, or pipe (default: comma)
    #[arg(long)]
    delimiter: Option<String>,

    /// Include length markers in arrays (default: true)
    #[arg(long, default_value_t = true)]
    length_marker: bool,

    /// Disable pretty-printing
    #[arg(long)]
    plain: bool,

    /// Maximum memory usage limit (e.g., 100MB, default: 100MB)
    #[arg(long)]
    memory_limit: Option<String>,

    /// Maximum processing time in seconds (default: 300)
    #[arg(long)]
    timeout: Option<u64>,

    /// Use SIMD-optimized JSON parser
    #[arg(long)]
    simd: bool,

    /// Only validate JSON, don't convert
    #[arg(long)]
    validate_only: bool,

    /// Output conversion statistics
    #[arg(long)]
    stats: bool,

    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,

    /// Suppress non-error output
    #[arg(long)]
    quiet: bool,

    /// Continue converting other files when one file fails
    #[arg(long)]
    continue_on_error: bool,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    // Set up logging
    if args.verbose {
        eprintln!("Verbose mode enabled");
    }

    // Create conversion configuration
    let config = create_conversion_config(&args)?;

    // Handle different input sources
    if args.validate_only {
        handle_validation(&args, &config)
    } else {
        handle_conversion(&args, &config)
    }
}

fn create_conversion_config(args: &CliArgs) -> Result<ConversionConfig> {
    let delimiter = match args.delimiter.as_deref() {
        Some("tab") => crate::conversion::DelimiterType::Tab,
        Some("pipe") => crate::conversion::DelimiterType::Pipe,
        Some("comma") | None => crate::conversion::DelimiterType::Comma,
        Some(other) => {
            return Err(anyhow::anyhow!(
                "Invalid delimiter '{}'. Use 'comma', 'tab', or 'pipe'",
                other
            ))
        }
    };

    let memory_limit = parse_memory_limit(&args.memory_limit)?;
    let timeout = Duration::from_secs(args.timeout.unwrap_or(300));

    Ok(ConversionConfig {
        indent_size: args.indent.unwrap_or(2),
        delimiter,
        length_marker: args.length_marker,
        quote_strings: crate::conversion::QuoteStrategy::Smart,
        memory_limit,
        timeout,
        enable_simd: args.simd,
        pretty: !args.plain,
        validate_output: true,
        include_schema: true,
        max_depth: Some(1000),
    })
}

fn parse_memory_limit(limit: &Option<String>) -> Result<usize> {
    match limit {
        None => Ok(100 * 1024 * 1024), // 100MB default
        Some(limit_str) => {
            // Parse memory limit string (e.g., "100MB", "1GB", "500KB")
            if limit_str.ends_with("MB") {
                let size = &limit_str[..limit_str.len() - 2];
                Ok(size
                    .parse::<f64>()
                    .map(|mb| (mb * 1024.0 * 1024.0) as usize)?)
            } else if limit_str.ends_with("KB") {
                let size = &limit_str[..limit_str.len() - 2];
                Ok(size.parse::<f64>().map(|kb| (kb * 1024.0) as usize)?)
            } else if limit_str.ends_with("GB") {
                let size = &limit_str[..limit_str.len() - 2];
                Ok(size
                    .parse::<f64>()
                    .map(|gb| (gb * 1024.0 * 1024.0 * 1024.0) as usize)?)
            } else if limit_str.ends_with("B") {
                let size = &limit_str[..limit_str.len() - 1];
                Ok(size.parse::<usize>()?)
            } else {
                // Assume bytes
                Ok(limit_str.parse::<usize>()?)
            }
        }
    }
}

fn handle_validation(args: &CliArgs, _config: &ConversionConfig) -> Result<()> {
    // Implementation for JSON validation only
    if args.stdin {
        let json_str = read_stdin()?;
        let _json_value = parse_json_validation(&json_str)?;
        if !args.quiet {
            println!("✓ Valid JSON");
        }
        Ok(())
    } else if let Some(input) = &args.input {
        let path = PathBuf::from(input);
        if path.is_file() {
            let json_str = std::fs::read_to_string(path)?;
            let _json_value = parse_json_validation(&json_str)?;
            if !args.quiet {
                println!("✓ Valid JSON");
            }
            Ok(())
        } else if path.is_dir() {
            // Validate all JSON files in directory
            validate_directory(&path, args.recursive)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Input path does not exist: {}", input))
        }
    } else {
        Err(anyhow::anyhow!(
            "No input provided. Use --stdin or provide an input path"
        ))
    }
}

fn handle_conversion(args: &CliArgs, config: &ConversionConfig) -> Result<()> {
    if args.stdin {
        convert_stdin(args, config)
    } else if let Some(input) = &args.input {
        let path = PathBuf::from(input);

        // Check if input looks like JSON string (starts with { or [)
        let trimmed = input.trim();
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            // Treat as JSON string
            convert_string(input, args, config)
        } else if path.is_file() {
            convert_file(&path, args, config)
        } else if path.is_dir() {
            convert_directory(&path, args, config)
        } else {
            Err(anyhow::anyhow!("Input path does not exist: {}", input))
        }
    } else {
        Err(anyhow::anyhow!(
            "No input provided. Use --stdin or provide an input path"
        ))
    }
}

fn convert_stdin(args: &CliArgs, config: &ConversionConfig) -> Result<()> {
    let json_str = read_stdin()?;
    convert_string(&json_str, args, config)
}

fn convert_file(input_path: &PathBuf, args: &CliArgs, config: &ConversionConfig) -> Result<()> {
    // Check file size before reading to avoid exhausting memory
    if let Ok(metadata) = std::fs::metadata(input_path) {
        if metadata.len() > config.memory_limit as u64 {
            return Err(anyhow::anyhow!(
                "JSON file too large: {} bytes (limit: {} bytes)",
                metadata.len(),
                config.memory_limit
            ));
        }
    }

    let json_str = std::fs::read_to_string(input_path)?;
    convert_string(&json_str, args, config)
}

fn convert_string(json_str: &str, args: &CliArgs, config: &ConversionConfig) -> Result<()> {
    // Parse JSON
    let json_source = JsonSource::String(json_str.to_string());
    let json_value = json_source.parse()?;

    // Convert to TOON
    let toon_data = convert_json_to_toon(&json_value, config)?;

    // Output result
    if let Some(output_path) = &args.output {
        // Write to file
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(output_path, &toon_data.content)?;

        if !args.quiet {
            println!("✓ Converted to: {}", output_path.display());
        }
    } else {
        // Write to stdout
        println!("{}", toon_data.content);
    }

    // Output statistics if requested
    if args.stats {
        output_statistics(&toon_data, args.quiet)?;
    }

    Ok(())
}

fn convert_single_file(
    input_path: &PathBuf,
    output_path: &PathBuf,
    config: &ConversionConfig,
) -> Result<()> {
    // Check file size before reading to avoid exhausting memory
    if let Ok(metadata) = std::fs::metadata(input_path) {
        if metadata.len() > config.memory_limit as u64 {
            return Err(anyhow::anyhow!(
                "JSON file too large: {} bytes (limit: {} bytes)",
                metadata.len(),
                config.memory_limit
            ));
        }
    }

    // Read JSON file
    let json_str = std::fs::read_to_string(input_path)?;

    // Parse JSON
    let json_source = JsonSource::String(json_str.to_string());
    let json_value = json_source.parse()?;

    // Convert to TOON
    let toon_data = convert_json_to_toon(&json_value, config)?;

    // Write to output file
    std::fs::write(output_path, &toon_data.content)?;

    Ok(())
}

fn convert_directory(input_dir: &PathBuf, args: &CliArgs, config: &ConversionConfig) -> Result<()> {
    let output_dir = args
        .output
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Output directory required for directory conversion"))?;

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Find all JSON files (use parser utilities)
    let json_files = crate::parser::directory::find_json_files(input_dir, args.recursive)
        .map_err(|e| anyhow::anyhow!("Failed finding JSON files: {}", e))?;

    if json_files.is_empty() {
        if !args.quiet {
            println!("No JSON files found in {}", input_dir.display());
        }
        return Ok(());
    }

    if !args.quiet {
        println!("Found {} JSON files", json_files.len());
    }

    // Process files
    for json_file in json_files {
        let relative_path = json_file
            .strip_prefix(input_dir)
            .map_err(|_| anyhow::anyhow!("Failed to get relative path"))?;

        let output_file = crate::cli::path_mapping::map_input_to_output(
            input_dir, &json_file, output_dir, "toon",
        );

        // Ensure output directory exists
        if let Some(parent) = output_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Read and convert file directly
        match convert_single_file(&json_file, &output_file, config) {
            Ok(_) => {
                if !args.quiet {
                    println!("✓ {} -> {}", relative_path.display(), output_file.display());
                }
            }
            Err(e) => {
                eprintln!("✗ Error converting {}: {}", relative_path.display(), e);
                if !args.continue_on_error {
                    return Err(anyhow::anyhow!("Aborting due to conversion error: {}", e));
                }
            }
        }
    }

    Ok(())
}

fn validate_directory(dir: &PathBuf, recursive: bool) -> Result<()> {
    let json_files = find_json_files(dir, recursive)?;

    for json_file in json_files {
        let relative_path = json_file
            .strip_prefix(dir)
            .map_err(|_| anyhow::anyhow!("Failed to get relative path"))?;

        match std::fs::read_to_string(&json_file) {
            Ok(content) => match parse_json_validation(&content) {
                Ok(_) => println!("✓ {}", relative_path.display()),
                Err(e) => eprintln!("✗ {}: {}", relative_path.display(), e),
            },
            Err(e) => eprintln!("✗ {}: {}", relative_path.display(), e),
        }
    }

    Ok(())
}

fn find_json_files(dir: &PathBuf, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut json_files = Vec::new();

    if recursive {
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                json_files.push(path.to_path_buf());
            }
        }
    } else {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                json_files.push(path);
            }
        }
    }

    Ok(json_files)
}

fn parse_json_validation(json_str: &str) -> Result<serde_json::Value> {
    serde_json::from_str(json_str).map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))
}

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

fn output_statistics(toon_data: &crate::conversion::ToonData, quiet: bool) -> Result<()> {
    if quiet {
        return Ok(());
    }

    println!("\nConversion Statistics:");
    println!("Input size: {} bytes", toon_data.metadata.input_size);
    println!("Output size: {} bytes", toon_data.metadata.output_size);
    println!(
        "Token reduction: {:.1}%",
        toon_data.metadata.token_reduction
    );
    println!(
        "Processing time: {}ms",
        toon_data.metadata.processing_time_ms
    );
    if toon_data.metadata.memory_peak_kb > 0 {
        println!("Peak memory: {} KB", toon_data.metadata.memory_peak_kb);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_convert_string_writes_file_and_creates_dirs() {
        let tmp = tempdir().unwrap();
        let output_path = tmp.path().join("nested/out.toon");

        let args = CliArgs {
            input: None,
            output: Some(output_path.clone()),
            stdin: false,
            recursive: false,
            indent: None,
            delimiter: None,
            length_marker: false,
            plain: false,
            memory_limit: None,
            timeout: None,
            simd: false,
            validate_only: false,
            stats: false,
            verbose: false,
            quiet: true,
            continue_on_error: false,
        };

        let json = r#"{"message": "hello"}"#;
        let cfg = create_conversion_config(&args).unwrap();

        // Convert - expect no error
        assert!(convert_string(json, &args, &cfg).is_ok());

        // Output file should have been created
        assert!(output_path.exists());
        let contents = fs::read_to_string(output_path).unwrap();
        assert!(!contents.is_empty());
    }

    #[test]
    fn test_convert_file_rejects_large_file() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("big.json");
        let mut f = fs::File::create(&file_path).unwrap();

        // Create a file larger than 1KB
        let payload = vec![b'a'; 2048];
        use std::io::Write;
        f.write_all(&payload).unwrap();

        let args = CliArgs {
            input: Some(file_path.to_string_lossy().to_string()),
            output: None,
            stdin: false,
            recursive: false,
            indent: None,
            delimiter: None,
            length_marker: false,
            plain: false,
            memory_limit: Some("1KB".to_string()),
            timeout: None,
            simd: false,
            validate_only: false,
            stats: false,
            verbose: false,
            quiet: true,
            continue_on_error: false,
        };

        let cfg = create_conversion_config(&args).unwrap();

        // convert_file uses metadata check which will fail due to limit
        if let Some(input) = &args.input {
            let path = PathBuf::from(input);
            let res = convert_file(&path, &args, &cfg);
            assert!(res.is_err());
        } else {
            panic!("Missing input path for test");
        }
    }
}
