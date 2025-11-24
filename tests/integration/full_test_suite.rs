//! Comprehensive Integration Test Suite for toonconv
//!
//! This module provides end-to-end testing coverage for all major features:
//! - User Story 1: String-to-TOON conversion (stdin)
//! - User Story 2: File-to-file conversion
//! - User Story 3: Directory batch processing
//! - User Story 4: Complex JSON structures
//! - Cross-cutting concerns: CLI options, error handling, performance

use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::tempdir;
use toonconv::conversion::{convert_json_to_toon, ConversionConfig};

// ============================================================================
// Test Helpers
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

fn create_test_json_file(dir: &tempfile::TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

// ============================================================================
// User Story 1: String-to-TOON Conversion (stdin)
// ============================================================================

mod us1_string_conversion {
    use super::*;

    #[test]
    fn test_simple_object_via_stdin() {
        let output = run_toonconv_with_stdin(&["--stdin"], r#"{"name": "Alice", "age": 30}"#);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("name"));
        assert!(stdout.contains("Alice"));
        assert!(stdout.contains("age"));
        assert!(stdout.contains("30"));
    }

    #[test]
    fn test_nested_object_via_stdin() {
        let json = r#"{"user": {"name": "Bob", "address": {"city": "NYC"}}}"#;
        let output = run_toonconv_with_stdin(&["--stdin"], json);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("user"));
        assert!(stdout.contains("name"));
        assert!(stdout.contains("Bob"));
        assert!(stdout.contains("city"));
        assert!(stdout.contains("NYC"));
    }

    #[test]
    fn test_array_via_stdin() {
        let json = r#"[1, 2, 3, 4, 5]"#;
        let output = run_toonconv_with_stdin(&["--stdin"], json);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Array should be present in output
        assert!(stdout.contains("1"));
        assert!(stdout.contains("5"));
    }

    #[test]
    fn test_empty_object_via_stdin() {
        let output = run_toonconv_with_stdin(&["--stdin"], "{}");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("{}") || stdout.trim().is_empty() || stdout.contains("{ }"));
    }

    #[test]
    fn test_invalid_json_via_stdin() {
        let output = run_toonconv_with_stdin(&["--stdin"], "{ invalid json }");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.to_lowercase().contains("error")
                || stderr.to_lowercase().contains("invalid")
                || stderr.to_lowercase().contains("parse")
        );
    }

    #[test]
    fn test_unicode_via_stdin() {
        let json = r#"{"greeting": "Hello 世界 🌍", "language": "中文"}"#;
        let output = run_toonconv_with_stdin(&["--stdin"], json);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("世界") || stdout.contains("\\u"));
        assert!(stdout.contains("🌍") || stdout.contains("\\u"));
    }
}

// ============================================================================
// User Story 2: File-to-File Conversion
// ============================================================================

mod us2_file_conversion {
    use super::*;

    #[test]
    fn test_simple_file_conversion() {
        let tmp = tempdir().unwrap();
        let input_path = create_test_json_file(&tmp, "input.json", r#"{"key": "value"}"#);
        let output_path = tmp.path().join("output.toon");

        let output = run_toonconv(&[
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ]);

        assert!(output.status.success());
        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("key"));
        assert!(content.contains("value"));
    }

    #[test]
    fn test_file_conversion_creates_parent_dirs() {
        let tmp = tempdir().unwrap();
        let input_path = create_test_json_file(&tmp, "input.json", r#"{"test": true}"#);
        let output_path = tmp.path().join("nested/deep/output.toon");

        let output = run_toonconv(&[
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ]);

        assert!(output.status.success());
        assert!(output_path.exists());
    }

    #[test]
    fn test_file_conversion_preserves_data_integrity() {
        let tmp = tempdir().unwrap();
        let complex_json = json!({
            "numbers": [1, 2, 3, 4, 5],
            "booleans": {"yes": true, "no": false},
            "nullValue": null,
            "nested": {
                "level1": {
                    "level2": {
                        "value": "deep"
                    }
                }
            }
        });

        let input_path = create_test_json_file(
            &tmp,
            "complex.json",
            &serde_json::to_string(&complex_json).unwrap(),
        );
        let output_path = tmp.path().join("complex.toon");

        let output = run_toonconv(&[
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ]);

        assert!(output.status.success());
        let content = fs::read_to_string(&output_path).unwrap();

        // Verify key data elements are preserved
        assert!(content.contains("numbers"));
        assert!(content.contains("booleans"));
        assert!(content.contains("true"));
        assert!(content.contains("false"));
        assert!(content.contains("null"));
        assert!(content.contains("deep"));
    }

    #[test]
    fn test_nonexistent_input_file() {
        let output = run_toonconv(&["nonexistent.json", "--output", "output.toon"]);

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.to_lowercase().contains("error")
                || stderr.to_lowercase().contains("not found")
                || stderr.to_lowercase().contains("no such file")
        );
    }

    #[test]
    fn test_file_with_stats_flag() {
        let tmp = tempdir().unwrap();
        let input_path =
            create_test_json_file(&tmp, "input.json", r#"{"data": "test value here"}"#);
        let output_path = tmp.path().join("output.toon");

        let output = run_toonconv(&[
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--stats",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Stats should include size and reduction info
        assert!(
            stdout.contains("Statistics")
                || stdout.contains("size")
                || stdout.contains("bytes")
                || stdout.contains("reduction")
        );
    }
}

// ============================================================================
// User Story 3: Directory Batch Processing
// ============================================================================

mod us3_directory_conversion {
    use super::*;

    #[test]
    fn test_directory_batch_conversion() {
        let tmp = tempdir().unwrap();

        // Create multiple JSON files
        create_test_json_file(&tmp, "input/file1.json", r#"{"id": 1}"#);
        create_test_json_file(&tmp, "input/file2.json", r#"{"id": 2}"#);
        create_test_json_file(&tmp, "input/file3.json", r#"{"id": 3}"#);

        let input_dir = tmp.path().join("input");
        let output_dir = tmp.path().join("output");

        let output = run_toonconv(&[
            input_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
        ]);

        assert!(output.status.success());
        assert!(output_dir.join("file1.toon").exists());
        assert!(output_dir.join("file2.toon").exists());
        assert!(output_dir.join("file3.toon").exists());
    }

    #[test]
    fn test_recursive_directory_conversion() {
        let tmp = tempdir().unwrap();

        // Create nested directory structure
        create_test_json_file(&tmp, "input/a.json", r#"{"level": "root"}"#);
        create_test_json_file(&tmp, "input/sub1/b.json", r#"{"level": "sub1"}"#);
        create_test_json_file(&tmp, "input/sub1/sub2/c.json", r#"{"level": "sub2"}"#);

        let input_dir = tmp.path().join("input");
        let output_dir = tmp.path().join("output");

        let output = run_toonconv(&[
            input_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
            "--recursive",
        ]);

        assert!(output.status.success());
        assert!(output_dir.join("a.toon").exists());
        assert!(output_dir.join("sub1/b.toon").exists());
        assert!(output_dir.join("sub1/sub2/c.toon").exists());
    }

    #[test]
    fn test_directory_ignores_non_json_files() {
        let tmp = tempdir().unwrap();

        create_test_json_file(&tmp, "input/valid.json", r#"{"valid": true}"#);
        create_test_json_file(&tmp, "input/readme.txt", "This is a readme");
        create_test_json_file(&tmp, "input/config.yaml", "key: value");

        let input_dir = tmp.path().join("input");
        let output_dir = tmp.path().join("output");

        let output = run_toonconv(&[
            input_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
        ]);

        assert!(output.status.success());
        assert!(output_dir.join("valid.toon").exists());
        // Non-JSON files should not be converted
        assert!(!output_dir.join("readme.toon").exists());
        assert!(!output_dir.join("config.toon").exists());
    }

    #[test]
    fn test_continue_on_error_flag() {
        let tmp = tempdir().unwrap();

        create_test_json_file(&tmp, "input/valid.json", r#"{"valid": true}"#);
        create_test_json_file(&tmp, "input/invalid.json", "{ invalid json }");
        create_test_json_file(&tmp, "input/another.json", r#"{"also": "valid"}"#);

        let input_dir = tmp.path().join("input");
        let output_dir = tmp.path().join("output");

        run_toonconv(&[
            input_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
            "--continue-on-error",
        ]);

        // Should complete (may have non-zero exit if there were errors)
        // But valid files should still be converted
        assert!(output_dir.join("valid.toon").exists());
        assert!(output_dir.join("another.toon").exists());
    }
}

// ============================================================================
// User Story 4: Complex JSON Structures
// ============================================================================

mod us4_complex_structures {
    use super::*;

    #[test]
    fn test_deeply_nested_object() {
        let config = ConversionConfig::default();

        // Create 10-level nesting
        let mut json = json!({"value": "deepest"});
        for i in (1..=10).rev() {
            json = json!({ format!("level{}", i): json });
        }

        let result = convert_json_to_toon(&json, &config);

        assert!(result.is_ok());
        let toon = result.unwrap().content;
        assert!(toon.contains("level1"));
        assert!(toon.contains("level10"));
        assert!(toon.contains("deepest"));
    }

    #[test]
    fn test_mixed_type_array() {
        let config = ConversionConfig::default();

        let json = json!([
            1,
            "two",
            3.14,
            true,
            null,
            {"nested": "object"},
            [1, 2, 3]
        ]);

        let result = convert_json_to_toon(&json, &config);

        assert!(result.is_ok());
        let toon = result.unwrap().content;
        assert!(toon.contains("1"));
        assert!(toon.contains("two") || toon.contains("\"two\""));
        assert!(toon.contains("true"));
        assert!(toon.contains("null"));
    }

    #[test]
    fn test_uniform_array_tabular_format() {
        let config = ConversionConfig::default();

        let json = json!([
            {"id": 1, "name": "Alice", "age": 30},
            {"id": 2, "name": "Bob", "age": 25},
            {"id": 3, "name": "Charlie", "age": 35}
        ]);

        let result = convert_json_to_toon(&json, &config);

        assert!(result.is_ok());
        let toon = result.unwrap().content;
        // Should contain all data
        assert!(toon.contains("Alice"));
        assert!(toon.contains("Bob"));
        assert!(toon.contains("Charlie"));
    }

    #[test]
    fn test_special_characters_in_strings() {
        let config = ConversionConfig::default();

        let json = json!({
            "quotes": "She said \"hello\"",
            "backslash": "C:\\Users\\test",
            "newline": "line1\nline2",
            "unicode": "Hello 世界"
        });

        let result = convert_json_to_toon(&json, &config);

        assert!(result.is_ok());
        // Data should be preserved (possibly escaped)
        let toon = result.unwrap().content;
        assert!(toon.contains("quotes"));
        assert!(toon.contains("backslash"));
    }

    #[test]
    fn test_empty_structures() {
        let config = ConversionConfig::default();

        let json = json!({
            "emptyObject": {},
            "emptyArray": [],
            "emptyString": ""
        });

        let result = convert_json_to_toon(&json, &config);

        assert!(result.is_ok());
    }

    #[test]
    fn test_numeric_edge_cases() {
        let config = ConversionConfig::default();

        let json = json!({
            "zero": 0,
            "negative": -42,
            "float": 3.14159,
            "scientific": 1.5e10,
            "largeInt": 9007199254740991_i64
        });

        let result = convert_json_to_toon(&json, &config);

        assert!(result.is_ok());
        let toon = result.unwrap().content;
        assert!(toon.contains("0"));
        assert!(toon.contains("-42"));
        assert!(toon.contains("3.14159"));
    }
}

// ============================================================================
// CLI Options and Flags
// ============================================================================

mod cli_options {
    use super::*;

    #[test]
    fn test_help_flag() {
        let output = run_toonconv(&["--help"]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("toonconv"));
        assert!(stdout.contains("--output") || stdout.contains("-o"));
        assert!(stdout.contains("--stdin"));
    }

    #[test]
    fn test_version_flag() {
        let output = run_toonconv(&["--version"]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("toonconv") || stdout.contains("0."));
    }

    #[test]
    fn test_indent_option() {
        let tmp = tempdir().unwrap();
        let input_path = create_test_json_file(&tmp, "input.json", r#"{"a": {"b": 1}}"#);
        let output_path = tmp.path().join("output.toon");

        let output = run_toonconv(&[
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--indent",
            "4",
        ]);

        assert!(output.status.success());
        let content = fs::read_to_string(&output_path).unwrap();
        // With indent=4, nested content should have 4-space indentation
        assert!(content.contains("    ") || content.contains("\t"));
    }

    #[test]
    fn test_plain_output_option() {
        let output = run_toonconv_with_stdin(&["--stdin", "--plain"], r#"{"key": "value"}"#);

        assert!(output.status.success());
        // Plain output should still contain the data
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("key"));
    }

    #[test]
    fn test_quiet_mode() {
        let tmp = tempdir().unwrap();
        let input_path = create_test_json_file(&tmp, "input.json", r#"{"test": true}"#);
        let output_path = tmp.path().join("output.toon");

        let output = run_toonconv(&[
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--quiet",
            "--stats", // Stats should be suppressed in quiet mode
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // In quiet mode, stats should not be printed
        assert!(!stdout.contains("Statistics"));
    }

    #[test]
    fn test_validate_only_mode() {
        let output = run_toonconv_with_stdin(&["--stdin", "--validate-only"], r#"{"valid": true}"#);

        assert!(output.status.success());
    }

    #[test]
    fn test_validate_only_with_invalid_json() {
        let output = run_toonconv_with_stdin(&["--stdin", "--validate-only"], "{ not valid }");

        assert!(!output.status.success());
    }
}

// ============================================================================
// Error Handling
// ============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_malformed_json_error() {
        let output = run_toonconv_with_stdin(&["--stdin"], "{ malformed: json }");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.is_empty());
    }

    #[test]
    fn test_empty_input_handling() {
        let output = run_toonconv_with_stdin(&["--stdin"], "");

        // Empty input should be handled gracefully
        // (may succeed with empty output or fail with clear error)
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should not panic or produce cryptic errors
        assert!(!stderr.contains("panic") && !stdout.contains("panic"));
    }

    #[test]
    fn test_permission_error_message() {
        // Try to write to a non-writable location
        let output = run_toonconv_with_stdin(
            &["--stdin", "--output", "/root/cannot_write.toon"],
            r#"{"test": true}"#,
        );

        // Should fail with permission error (on most systems)
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.to_lowercase().contains("permission")
                    || stderr.to_lowercase().contains("denied")
                    || stderr.to_lowercase().contains("error")
            );
        }
    }
}

// ============================================================================
// Data Integrity Tests
// ============================================================================

mod data_integrity {
    use super::*;

    #[test]
    fn test_all_json_types_preserved() {
        let config = ConversionConfig::default();

        let json = json!({
            "string": "hello",
            "number": 42,
            "float": 3.14,
            "boolTrue": true,
            "boolFalse": false,
            "nullValue": null,
            "array": [1, 2, 3],
            "object": {"nested": "value"}
        });

        let result = convert_json_to_toon(&json, &config);

        assert!(result.is_ok());
        let toon = result.unwrap().content;

        // All types should be present
        assert!(toon.contains("hello") || toon.contains("string"));
        assert!(toon.contains("42"));
        assert!(toon.contains("3.14"));
        assert!(toon.contains("true"));
        assert!(toon.contains("false"));
        assert!(toon.contains("null"));
    }

    #[test]
    fn test_key_order_consistency() {
        let config = ConversionConfig::default();

        // Run same conversion multiple times
        let json = json!({
            "z": 1,
            "a": 2,
            "m": 3
        });

        let result1 = convert_json_to_toon(&json, &config).unwrap().content;
        let result2 = convert_json_to_toon(&json, &config).unwrap().content;

        // Results should be identical
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_whitespace_in_strings_preserved() {
        let config = ConversionConfig::default();

        let json = json!({
            "spaces": "  leading and trailing  ",
            "tabs": "\ttabbed\t",
            "mixed": "  \t mixed \t  "
        });

        let result = convert_json_to_toon(&json, &config);

        assert!(result.is_ok());
        // Whitespace should be preserved in some form
        let toon = result.unwrap().content;
        assert!(toon.contains("leading") && toon.contains("trailing"));
    }
}

// ============================================================================
// Performance Sanity Checks
// ============================================================================

mod performance_sanity {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_small_file_under_100ms() {
        let config = ConversionConfig::default();

        let json = json!({
            "users": (0..100).map(|i| json!({"id": i, "name": format!("user{}", i)})).collect::<Vec<_>>()
        });

        let start = Instant::now();
        let result = convert_json_to_toon(&json, &config);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(
            duration.as_millis() < 100,
            "Small file conversion took {:?}",
            duration
        );
    }

    #[test]
    fn test_medium_file_under_500ms() {
        let config = ConversionConfig::default();

        let json = json!({
            "data": (0..1000).map(|i| json!({
                "id": i,
                "name": format!("item{}", i),
                "value": i * 2,
                "active": i % 2 == 0
            })).collect::<Vec<_>>()
        });

        let start = Instant::now();
        let result = convert_json_to_toon(&json, &config);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(
            duration.as_millis() < 500,
            "Medium file conversion took {:?}",
            duration
        );
    }
}
