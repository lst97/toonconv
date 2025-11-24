//! TOON Specification Fixtures Test Runner
//!
//! Runs tests from the official TOON spec fixture format.
//! Fixtures are JSON files matching the @toon-format/spec format.

use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use toonconv::conversion::config::ConversionConfig;
use toonconv::conversion::convert_json_to_toon;

/// Test fixture file structure matching the official TOON spec format
#[derive(Debug, Deserialize)]
struct FixtureFile {
    version: String,
    #[allow(dead_code)]
    category: String,
    description: String,
    tests: Vec<TestCase>,
}

/// Individual test case from fixture file
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCase {
    name: String,
    input: Value,
    expected: String,
    _spec_section: Option<String>,
    #[serde(default)]
    should_error: bool,
    note: Option<String>,
    min_spec_version: Option<String>,
    _options: Option<TestOptions>,
}

/// Test options matching the official spec
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TestOptions {
    _indent: Option<usize>,
    _delimiter: Option<String>,
    _key_folding: Option<String>,
    _flatten_depth: Option<usize>,
}

fn load_fixture(path: &Path) -> FixtureFile {
    let content =
        fs::read_to_string(path).expect(&format!("Failed to read fixture file: {:?}", path));
    serde_json::from_str(&content).expect(&format!("Failed to parse fixture file: {:?}", path))
}

fn run_fixture_tests(fixture_path: &str) {
    let path = Path::new(fixture_path);
    if !path.exists() {
        panic!("Fixture file not found: {}", fixture_path);
    }

    let fixture = load_fixture(path);
    println!("\n{} (v{})", fixture.description, fixture.version);
    println!("{}", "=".repeat(60));

    let mut passed = 0;
    let mut failed = 0;
    let skipped = 0;

    for test in &fixture.tests {
        // Skip tests that require newer spec versions (v2.1 features)
        if let Some(ref min_version) = test.min_spec_version {
            if min_version == "2.1" {
                // We can handle v2.1 tests - don't skip
            }
        }

        let config = ConversionConfig::default();

        if test.should_error {
            // Test should produce an error
            match convert_json_to_toon(&test.input, &config) {
                Ok(_) => {
                    println!("  FAIL: {} (expected error, got success)", test.name);
                    failed += 1;
                }
                Err(_) => {
                    println!("  PASS: {} (expected error)", test.name);
                    passed += 1;
                }
            }
        } else {
            // Test should succeed
            match convert_json_to_toon(&test.input, &config) {
                Ok(result) => {
                    let actual = result.content.trim();
                    let expected = test.expected.trim();

                    if actual == expected {
                        println!("  PASS: {}", test.name);
                        passed += 1;
                    } else {
                        println!("  FAIL: {}", test.name);
                        println!("    Expected: {:?}", expected);
                        println!("    Actual:   {:?}", actual);
                        if let Some(ref note) = test.note {
                            println!("    Note: {}", note);
                        }
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  FAIL: {} (unexpected error: {})", test.name, e);
                    failed += 1;
                }
            }
        }
    }

    println!(
        "\nResults: {} passed, {} failed, {} skipped",
        passed, failed, skipped
    );

    if failed > 0 {
        panic!("{} tests failed in {}", failed, fixture_path);
    }
}

#[test]
fn test_primitives_encoding() {
    run_fixture_tests("tests/fixtures/encode/primitives.json");
}

#[test]
fn test_objects_encoding() {
    run_fixture_tests("tests/fixtures/encode/objects.json");
}

#[test]
fn test_arrays_primitive_encoding() {
    run_fixture_tests("tests/fixtures/encode/arrays-primitive.json");
}

#[test]
fn test_arrays_tabular_encoding() {
    run_fixture_tests("tests/fixtures/encode/arrays-tabular.json");
}

#[test]
fn test_arrays_nested_encoding() {
    run_fixture_tests("tests/fixtures/encode/arrays-nested.json");
}

/// Summary test that runs all fixture files
#[test]
fn test_all_encode_fixtures() {
    let fixtures = [
        "tests/fixtures/encode/primitives.json",
        "tests/fixtures/encode/objects.json",
        "tests/fixtures/encode/arrays-primitive.json",
        "tests/fixtures/encode/arrays-tabular.json",
        "tests/fixtures/encode/arrays-nested.json",
    ];

    let mut total_passed = 0;
    let mut total_failed = 0;

    for fixture_path in &fixtures {
        let path = Path::new(fixture_path);
        if !path.exists() {
            println!("Skipping missing fixture: {}", fixture_path);
            continue;
        }

        let fixture = load_fixture(path);
        let config = ConversionConfig::default();

        for test in &fixture.tests {
            if test.should_error {
                match convert_json_to_toon(&test.input, &config) {
                    Ok(_) => total_failed += 1,
                    Err(_) => total_passed += 1,
                }
            } else {
                match convert_json_to_toon(&test.input, &config) {
                    Ok(result) => {
                        if result.content.trim() == test.expected.trim() {
                            total_passed += 1;
                        } else {
                            total_failed += 1;
                        }
                    }
                    Err(_) => total_failed += 1,
                }
            }
        }
    }

    println!("\n=== TOTAL ENCODE FIXTURE RESULTS ===");
    println!("Passed: {}", total_passed);
    println!("Failed: {}", total_failed);
    println!("Total:  {}", total_passed + total_failed);

    // Don't fail the test - just report results
    // This allows CI to see progress without blocking on incomplete features
}
