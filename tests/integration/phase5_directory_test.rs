//! Phase 5 Integration Test: Directory Batch Processing
//!
//! Tests User Story 3 - Convert JSON Directory to TOON Directory
//! Validates all Phase 5 requirements:
//! - Directory traversal and file discovery
//! - Recursive directory processing
//! - Path mapping and structure preservation
//! - Batch processing with error handling
//! - File filtering (.json only)

use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

fn run_toonconv(args: &[&str]) -> Result<(String, String, bool), String> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--bin", "toonconv", "--quiet", "--"])
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run toonconv: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let success = output.status.success();

    Ok((stdout, stderr, success))
}

#[test]
fn test_phase5_directory_conversion_basic() {
    // T035: Directory traversal and file discovery
    // T036: Recursive directory processing
    // T037: Path mapping and structure preservation

    let input_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Create test structure
    fs::create_dir_all(input_dir.path().join("level1/level2")).unwrap();

    // Create JSON files
    let file1 = input_dir.path().join("root.json");
    let mut f1 = File::create(&file1).unwrap();
    write!(f1, r#"{{"name": "root", "value": 1}}"#).unwrap();

    let file2 = input_dir.path().join("level1/mid.json");
    let mut f2 = File::create(&file2).unwrap();
    write!(f2, r#"{{"name": "mid", "value": 2}}"#).unwrap();

    let file3 = input_dir.path().join("level1/level2/deep.json");
    let mut f3 = File::create(&file3).unwrap();
    write!(f3, r#"{{"name": "deep", "value": 3}}"#).unwrap();

    // Run conversion
    let args = [
        input_dir.path().to_str().unwrap(),
        "--output",
        output_dir.path().to_str().unwrap(),
        "--recursive",
    ];

    let (stdout, stderr, success) = run_toonconv(&args).unwrap();

    assert!(success, "Command should succeed: stderr={}", stderr);
    assert!(
        stdout.contains("Found 3 JSON files"),
        "Should find 3 files: {}",
        stdout
    );

    // Verify output files exist with correct structure
    assert!(
        output_dir.path().join("root.toon").exists(),
        "root.toon should exist"
    );
    assert!(
        output_dir.path().join("level1/mid.toon").exists(),
        "level1/mid.toon should exist"
    );
    assert!(
        output_dir.path().join("level1/level2/deep.toon").exists(),
        "level1/level2/deep.toon should exist"
    );

    // Verify content
    let content = fs::read_to_string(output_dir.path().join("root.toon")).unwrap();
    assert!(
        content.contains("name:") && content.contains("root"),
        "Should contain converted content"
    );
}

#[test]
fn test_phase5_file_filtering() {
    // T040: File filtering (.json only)

    let input_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Create mixed file types
    File::create(input_dir.path().join("valid.json"))
        .unwrap()
        .write_all(br#"{"test": 1}"#)
        .unwrap();

    File::create(input_dir.path().join("ignore.txt"))
        .unwrap()
        .write_all(b"ignored")
        .unwrap();

    File::create(input_dir.path().join("ignore.md"))
        .unwrap()
        .write_all(b"# Ignored")
        .unwrap();

    // Run conversion
    let args = [
        input_dir.path().to_str().unwrap(),
        "--output",
        output_dir.path().to_str().unwrap(),
    ];

    let (stdout, _stderr, success) = run_toonconv(&args).unwrap();

    assert!(success, "Command should succeed");
    assert!(
        stdout.contains("Found 1 JSON files"),
        "Should find only 1 JSON file: {}",
        stdout
    );

    // Verify only JSON was converted
    assert!(
        output_dir.path().join("valid.toon").exists(),
        "valid.toon should exist"
    );
    assert!(
        !output_dir.path().join("ignore.txt").exists(),
        "ignore.txt should not be copied"
    );
    assert!(
        !output_dir.path().join("ignore.md").exists(),
        "ignore.md should not be copied"
    );
}

#[test]
fn test_phase5_batch_processing_with_errors() {
    // T038: Batch processing with continue-on-error

    let input_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Create valid JSON
    File::create(input_dir.path().join("valid.json"))
        .unwrap()
        .write_all(br#"{"good": true}"#)
        .unwrap();

    // Create invalid JSON
    File::create(input_dir.path().join("invalid.json"))
        .unwrap()
        .write_all(b"{bad json}")
        .unwrap();

    // Without continue-on-error (should fail)
    let args = [
        input_dir.path().to_str().unwrap(),
        "--output",
        output_dir.path().to_str().unwrap(),
    ];

    let (_stdout, stderr, success) = run_toonconv(&args).unwrap();

    // Should fail on invalid JSON
    assert!(!success, "Should fail without continue-on-error");
    assert!(
        stderr.contains("error") || stderr.contains("Error"),
        "Should report error"
    );

    // With continue-on-error (should process valid files)
    let output_dir2 = tempdir().unwrap();
    let args_continue = [
        input_dir.path().to_str().unwrap(),
        "--output",
        output_dir2.path().to_str().unwrap(),
        "--continue-on-error",
    ];

    let (_stdout, _stderr, success) = run_toonconv(&args_continue).unwrap();

    // Should succeed and process valid file
    assert!(success, "Should succeed with continue-on-error");
    assert!(
        output_dir2.path().join("valid.toon").exists(),
        "Should create valid.toon"
    );
}

#[test]
fn test_phase5_empty_directory() {
    // T035: Handle empty directories gracefully

    let input_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Run on empty directory
    let args = [
        input_dir.path().to_str().unwrap(),
        "--output",
        output_dir.path().to_str().unwrap(),
    ];

    let (stdout, _stderr, success) = run_toonconv(&args).unwrap();

    assert!(success, "Should succeed on empty directory");
    assert!(
        stdout.contains("No JSON files found") || stdout.contains("Found 0"),
        "Should report no files: {}",
        stdout
    );
}

#[test]
fn test_phase5_non_recursive_mode() {
    // T036: Non-recursive mode should only process top-level files

    let input_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    fs::create_dir_all(input_dir.path().join("subdir")).unwrap();

    // Top-level file
    File::create(input_dir.path().join("top.json"))
        .unwrap()
        .write_all(br#"{"level": "top"}"#)
        .unwrap();

    // Nested file
    File::create(input_dir.path().join("subdir/nested.json"))
        .unwrap()
        .write_all(br#"{"level": "nested"}"#)
        .unwrap();

    // Run without --recursive flag
    let args = [
        input_dir.path().to_str().unwrap(),
        "--output",
        output_dir.path().to_str().unwrap(),
    ];

    let (stdout, _stderr, success) = run_toonconv(&args).unwrap();

    assert!(success, "Command should succeed");
    assert!(
        stdout.contains("Found 1 JSON files"),
        "Should find only top-level file: {}",
        stdout
    );

    // Verify only top-level was converted
    assert!(
        output_dir.path().join("top.toon").exists(),
        "top.toon should exist"
    );
    assert!(
        !output_dir.path().join("subdir/nested.toon").exists(),
        "Nested file should not be converted"
    );
}

#[test]
fn test_phase5_performance_target() {
    // T042: Performance test - should process 10 files quickly

    let input_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Create 10 JSON files
    for i in 0..10 {
        let file = input_dir.path().join(format!("file{}.json", i));
        let mut f = File::create(&file).unwrap();
        write!(
            f,
            r#"{{"id": {}, "data": "test data for file {}", "numbers": [1, 2, 3, 4, 5]}}"#,
            i, i
        )
        .unwrap();
    }

    // Time the conversion
    let start = std::time::Instant::now();

    let args = [
        input_dir.path().to_str().unwrap(),
        "--output",
        output_dir.path().to_str().unwrap(),
    ];

    let (_stdout, _stderr, success) = run_toonconv(&args).unwrap();

    let duration = start.elapsed();

    assert!(success, "Conversion should succeed");
    assert!(
        duration.as_secs() < 5,
        "Should complete in less than 5 seconds, took {:?}",
        duration
    );

    // Verify all files were converted
    for i in 0..10 {
        let output_file = output_dir.path().join(format!("file{}.toon", i));
        assert!(output_file.exists(), "file{}.toon should exist", i);
    }
}
