//! Usability Test Harness for toonconv (SC-007 Validation)
//!
//! This module validates the usability requirements from the specification:
//! - Clear error messages
//! - Intuitive CLI interface
//! - Helpful output formatting
//! - Graceful degradation
//! - User-friendly defaults
//!
//! These tests simulate real user scenarios and validate the user experience.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::tempdir;

// ============================================================================
// Test Infrastructure
// ============================================================================

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("toonconv");
    path
}

fn run_toonconv(args: &[&str]) -> std::process::Output {
    Command::new(get_binary_path())
        .args(args)
        .output()
        .expect("Failed to execute toonconv")
}

fn run_toonconv_with_stdin(args: &[&str], stdin_data: &str) -> std::process::Output {
    let mut child = Command::new(get_binary_path())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn toonconv");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(stdin_data.as_bytes())
            .expect("Failed to write to stdin");
    }

    child.wait_with_output().expect("Failed to wait on child")
}

fn create_test_file(dir: &tempfile::TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

// ============================================================================
// Scenario 1: First-Time User Experience
// ============================================================================

mod first_time_user {
    use super::*;

    /// A new user runs toonconv without arguments - should show helpful usage info
    #[test]
    fn scenario_no_arguments_shows_help() {
        let output = run_toonconv(&[]);

        // Should either show help or a clear error message
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        // Must mention how to use the tool
        assert!(
            combined.to_lowercase().contains("usage")
                || combined.to_lowercase().contains("help")
                || combined.contains("--stdin")
                || combined.contains("--output")
                || combined.contains("toonconv"),
            "No arguments should show usage info. Got: {}",
            combined
        );
    }

    /// User asks for help - should get comprehensive documentation
    #[test]
    fn scenario_help_is_comprehensive() {
        let output = run_toonconv(&["--help"]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Help should explain key features
        let checks = vec![
            ("--stdin", "stdin flag should be documented"),
            ("--output", "output flag should be documented"),
            ("JSON", "should mention JSON"),
            ("TOON", "should mention TOON"),
        ];

        for (term, msg) in checks {
            assert!(
                stdout.contains(term) || stdout.to_lowercase().contains(&term.to_lowercase()),
                "{}: Help output doesn't contain '{}'. Help: {}",
                msg,
                term,
                stdout
            );
        }
    }

    /// User checks version - should get clear version info
    #[test]
    fn scenario_version_is_clear() {
        let output = run_toonconv(&["--version"]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should contain version number pattern
        assert!(
            stdout.contains("0.") || stdout.contains("1.") || stdout.contains("toonconv"),
            "Version output unclear: {}",
            stdout
        );
    }
}

// ============================================================================
// Scenario 2: Common Use Cases
// ============================================================================

mod common_use_cases {
    use super::*;

    /// User wants to quickly convert a JSON string from command line
    #[test]
    fn scenario_quick_string_conversion() {
        let output = run_toonconv_with_stdin(&["--stdin"], r#"{"message": "Hello, World!"}"#);

        assert!(
            output.status.success(),
            "Quick conversion failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("message") && stdout.contains("Hello"),
            "Output should contain the converted data: {}",
            stdout
        );
    }

    /// User converts a file and specifies output location
    #[test]
    fn scenario_file_to_file_conversion() {
        let tmp = tempdir().unwrap();
        let input = create_test_file(&tmp, "data.json", r#"{"user": "Alice", "role": "admin"}"#);
        let output_path = tmp.path().join("result.toon");

        let output = run_toonconv(&[
            input.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ]);

        assert!(
            output.status.success(),
            "File conversion failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output_path.exists(), "Output file was not created");

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(
            content.contains("user") && content.contains("Alice"),
            "Output file content incorrect: {}",
            content
        );
    }

    /// User wants to convert an entire directory of JSON files
    #[test]
    fn scenario_batch_directory_conversion() {
        let tmp = tempdir().unwrap();

        // Create sample JSON files
        create_test_file(&tmp, "input/users.json", r#"[{"id": 1}, {"id": 2}]"#);
        create_test_file(&tmp, "input/config.json", r#"{"debug": false}"#);
        create_test_file(&tmp, "input/data.json", r#"{"items": []}"#);

        let input_dir = tmp.path().join("input");
        let output_dir = tmp.path().join("output");

        let output = run_toonconv(&[
            input_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
        ]);

        assert!(
            output.status.success(),
            "Batch conversion failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // All files should be converted
        assert!(output_dir.join("users.toon").exists(), "users.toon missing");
        assert!(
            output_dir.join("config.toon").exists(),
            "config.toon missing"
        );
        assert!(output_dir.join("data.toon").exists(), "data.toon missing");
    }

    /// User wants to validate JSON without converting
    #[test]
    fn scenario_validation_only() {
        // Valid JSON
        let output = run_toonconv_with_stdin(&["--stdin", "--validate-only"], r#"{"valid": true}"#);
        assert!(output.status.success(), "Valid JSON should pass validation");

        // Invalid JSON
        let output = run_toonconv_with_stdin(&["--stdin", "--validate-only"], "{ not valid }");
        assert!(
            !output.status.success(),
            "Invalid JSON should fail validation"
        );
    }
}

// ============================================================================
// Scenario 3: Error Handling and Messages
// ============================================================================

mod error_handling {
    use super::*;

    /// Invalid JSON should produce a helpful error message
    #[test]
    fn scenario_invalid_json_error_message() {
        let output = run_toonconv_with_stdin(&["--stdin"], "{ this is: not valid JSON }");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Error should be descriptive, not cryptic
        assert!(
            stderr.to_lowercase().contains("error")
                || stderr.to_lowercase().contains("invalid")
                || stderr.to_lowercase().contains("parse")
                || stderr.to_lowercase().contains("json"),
            "Error message should mention the problem: {}",
            stderr
        );

        // Should not contain stack traces or internal details for simple errors
        assert!(
            !stderr.contains("panic") && !stderr.contains("RUST_BACKTRACE"),
            "Error message should not contain internal details: {}",
            stderr
        );
    }

    /// File not found should produce a clear error
    #[test]
    fn scenario_file_not_found_error() {
        let output = run_toonconv(&["nonexistent_file_12345.json"]);

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            stderr.to_lowercase().contains("not found")
                || stderr.to_lowercase().contains("no such file")
                || stderr.to_lowercase().contains("does not exist")
                || stderr.to_lowercase().contains("error"),
            "Should indicate file not found: {}",
            stderr
        );
    }

    /// Empty input should be handled gracefully
    #[test]
    fn scenario_empty_input_handling() {
        let output = run_toonconv_with_stdin(&["--stdin"], "");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should not panic
        assert!(
            !stderr.contains("panic") && !stdout.contains("panic"),
            "Empty input should not cause panic"
        );

        // If it fails, should have a clear message
        if !output.status.success() {
            assert!(
                stderr.to_lowercase().contains("empty")
                    || stderr.to_lowercase().contains("error")
                    || stderr.to_lowercase().contains("input"),
                "Error for empty input should be clear: {}",
                stderr
            );
        }
    }

    /// Invalid CLI options should produce helpful suggestions
    #[test]
    fn scenario_invalid_option_error() {
        let output = run_toonconv(&["--nonexistent-option"]);

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should mention the invalid option or suggest alternatives
        assert!(
            stderr.contains("nonexistent")
                || stderr.to_lowercase().contains("unknown")
                || stderr.to_lowercase().contains("unrecognized")
                || stderr.to_lowercase().contains("error"),
            "Should identify invalid option: {}",
            stderr
        );
    }
}

// ============================================================================
// Scenario 4: Output Quality
// ============================================================================

mod output_quality {
    use super::*;

    /// Output should be readable and properly formatted
    #[test]
    fn scenario_output_is_readable() {
        let json = r#"{
            "user": {
                "name": "Alice",
                "email": "alice@example.com"
            },
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        }"#;

        let output = run_toonconv_with_stdin(&["--stdin"], json);
        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Output should have some structure (newlines or indentation)
        assert!(
            stdout.contains('\n') || stdout.contains("  "),
            "Output should be formatted for readability: {}",
            stdout
        );

        // All key data should be present
        assert!(stdout.contains("user"), "Missing 'user' in output");
        assert!(stdout.contains("Alice"), "Missing 'Alice' in output");
        assert!(stdout.contains("settings"), "Missing 'settings' in output");
    }

    /// Statistics output should be informative
    #[test]
    fn scenario_stats_are_informative() {
        let tmp = tempdir().unwrap();
        let input = create_test_file(
            &tmp,
            "data.json",
            r#"{"data": "some test content here for statistics"}"#,
        );
        let output_path = tmp.path().join("result.toon");

        let output = run_toonconv(&[
            input.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--stats",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Stats should include useful metrics
        let has_stats = stdout.to_lowercase().contains("size")
            || stdout.to_lowercase().contains("bytes")
            || stdout.to_lowercase().contains("reduction")
            || stdout.to_lowercase().contains("time")
            || stdout.to_lowercase().contains("statistics");

        assert!(has_stats, "Stats output should be informative: {}", stdout);
    }
}

// ============================================================================
// Scenario 5: Workflow Integration
// ============================================================================

mod workflow_integration {
    use super::*;

    /// Tool should work well in pipelines
    #[test]
    fn scenario_pipeline_friendly() {
        // Simulate: echo '{"a":1}' | toonconv --stdin | grep "a"
        let output = run_toonconv_with_stdin(&["--stdin"], r#"{"searchable": "value"}"#);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Output should be grep-able (contain the data on stdout)
        assert!(
            stdout.contains("searchable"),
            "Output should be pipeline-friendly"
        );
    }

    /// Quiet mode should suppress non-essential output
    #[test]
    fn scenario_quiet_mode_works() {
        let tmp = tempdir().unwrap();
        let input = create_test_file(&tmp, "data.json", r#"{"quiet": "test"}"#);
        let output_path = tmp.path().join("result.toon");

        let output = run_toonconv(&[
            input.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--quiet",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // In quiet mode, there should be minimal output
        let total_output = format!("{}{}", stdout.trim(), stderr.trim());
        assert!(
            total_output.is_empty() || total_output.len() < 100,
            "Quiet mode should minimize output. Got: {}",
            total_output
        );
    }

    /// Multiple files with continue-on-error should process all valid files
    #[test]
    fn scenario_continue_on_error_processes_all() {
        let tmp = tempdir().unwrap();

        create_test_file(&tmp, "input/good1.json", r#"{"id": 1}"#);
        create_test_file(&tmp, "input/bad.json", "{ invalid }");
        create_test_file(&tmp, "input/good2.json", r#"{"id": 2}"#);

        let input_dir = tmp.path().join("input");
        let output_dir = tmp.path().join("output");

        let _output = run_toonconv(&[
            input_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
            "--continue-on-error",
        ]);

        // Good files should be converted despite bad file
        assert!(
            output_dir.join("good1.toon").exists(),
            "good1.toon should exist"
        );
        assert!(
            output_dir.join("good2.toon").exists(),
            "good2.toon should exist"
        );
    }
}

// ============================================================================
// Scenario 6: Edge Cases Users Might Encounter
// ============================================================================

mod edge_cases {
    use super::*;

    /// Unicode content should be handled properly
    #[test]
    fn scenario_unicode_handling() {
        let json = r#"{"greeting": "Hello 世界!", "emoji": "🎉🚀💻"}"#;
        let output = run_toonconv_with_stdin(&["--stdin"], json);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Unicode should be preserved or properly escaped
        assert!(
            stdout.contains("世界") || stdout.contains("\\u"),
            "Unicode should be handled: {}",
            stdout
        );
    }

    /// Very long strings should not cause issues
    #[test]
    fn scenario_long_strings() {
        let long_value = "x".repeat(10000);
        let json = format!(r#"{{"long": "{}"}}"#, long_value);

        let output = run_toonconv_with_stdin(&["--stdin"], &json);

        assert!(
            output.status.success(),
            "Long strings should be handled: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Deeply nested structures should not crash
    #[test]
    fn scenario_deep_nesting() {
        // Create 20-level nesting
        let mut json = r#"{"value": "deep"}"#.to_string();
        for _ in 0..20 {
            json = format!(r#"{{"nested": {}}}"#, json);
        }

        let output = run_toonconv_with_stdin(&["--stdin"], &json);

        assert!(
            output.status.success(),
            "Deep nesting should be handled: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Special characters in keys and values
    #[test]
    fn scenario_special_characters() {
        let json = r#"{
            "with spaces": "value",
            "with:colon": "value",
            "with\"quote": "value",
            "normal": "line1\nline2\ttabbed"
        }"#;

        let output = run_toonconv_with_stdin(&["--stdin"], json);

        assert!(
            output.status.success(),
            "Special characters should be handled: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

// ============================================================================
// Scenario 7: Performance Expectations
// ============================================================================

mod performance_expectations {
    use super::*;
    use std::time::Instant;

    /// Small files should convert quickly (user expectation: instant)
    #[test]
    fn scenario_small_file_feels_instant() {
        let json = r#"{"small": "file", "count": 42}"#;

        let start = Instant::now();
        let output = run_toonconv_with_stdin(&["--stdin"], json);
        let duration = start.elapsed();

        assert!(output.status.success());
        assert!(
            duration.as_millis() < 1000,
            "Small file should feel instant (<1s), took {:?}",
            duration
        );
    }

    /// Medium-sized data should complete in reasonable time
    #[test]
    fn scenario_medium_data_reasonable_time() {
        // Generate ~100KB of JSON
        let items: Vec<String> = (0..500)
            .map(|i| {
                format!(
                    r#"{{"id": {}, "name": "item_{}", "value": {}}}"#,
                    i,
                    i,
                    i * 10
                )
            })
            .collect();
        let json = format!("[{}]", items.join(","));

        let start = Instant::now();
        let output = run_toonconv_with_stdin(&["--stdin"], &json);
        let duration = start.elapsed();

        assert!(output.status.success());
        assert!(
            duration.as_secs() < 5,
            "Medium data should complete in <5s, took {:?}",
            duration
        );
    }
}

// ============================================================================
// Summary: Usability Checklist
// ============================================================================

/// This test documents the usability requirements being validated
#[test]
fn usability_checklist_documentation() {
    println!("=== Usability Test Coverage (SC-007) ===");
    println!();
    println!("1. First-Time User Experience:");
    println!("   - [x] No arguments shows helpful usage");
    println!("   - [x] --help provides comprehensive documentation");
    println!("   - [x] --version shows clear version info");
    println!();
    println!("2. Common Use Cases:");
    println!("   - [x] Quick string conversion via stdin");
    println!("   - [x] File-to-file conversion");
    println!("   - [x] Batch directory conversion");
    println!("   - [x] Validation-only mode");
    println!();
    println!("3. Error Handling:");
    println!("   - [x] Invalid JSON produces helpful errors");
    println!("   - [x] File not found has clear message");
    println!("   - [x] Empty input handled gracefully");
    println!("   - [x] Invalid options identified");
    println!();
    println!("4. Output Quality:");
    println!("   - [x] Output is readable and formatted");
    println!("   - [x] Statistics are informative");
    println!();
    println!("5. Workflow Integration:");
    println!("   - [x] Pipeline-friendly output");
    println!("   - [x] Quiet mode works");
    println!("   - [x] Continue-on-error processes all valid files");
    println!();
    println!("6. Edge Cases:");
    println!("   - [x] Unicode handling");
    println!("   - [x] Long strings");
    println!("   - [x] Deep nesting");
    println!("   - [x] Special characters");
    println!();
    println!("7. Performance:");
    println!("   - [x] Small files feel instant");
    println!("   - [x] Medium data completes reasonably");
    println!();
    println!("=== All usability scenarios covered ===");
}
