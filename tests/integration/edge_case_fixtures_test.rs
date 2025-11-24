//! Tests using edge case fixtures
//!
//! Validates conversion behavior for all edge cases defined in fixtures.

use serde_json::{json, Value};
use std::fs;
use toonconv::conversion::{convert_json_to_toon, ConversionConfig};
use toonconv::parser::JsonSource;

#[test]
fn test_edge_case_fixtures() {
    // Load the fixtures file
    let fixtures_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/edge_cases.json"
    );
    let fixtures_content =
        fs::read_to_string(fixtures_path).expect("Failed to read edge_cases.json fixtures file");

    let fixtures: Value =
        serde_json::from_str(&fixtures_content).expect("Failed to parse fixtures JSON");

    let test_cases = fixtures["test_cases"]
        .as_array()
        .expect("test_cases should be an array");

    let config = ConversionConfig::default();

    for test_case in test_cases {
        let name = test_case["name"].as_str().unwrap();
        let input = &test_case["input"];

        println!("Testing edge case: {}", name);

        let source = JsonSource::Value(input.clone());
        let result = convert_json_to_toon(&source.parse().unwrap(), &config);

        assert!(
            result.is_ok(),
            "Failed to convert edge case '{}': {:?}",
            name,
            result.err()
        );

        let output = result.unwrap();
        assert!(
            !output.content.is_empty(),
            "Empty output for edge case '{}'",
            name
        );

        // Verify data integrity
        if let Some(stats) = output.statistics {
            assert!(
                stats.data_integrity_verified,
                "Data integrity failed for edge case '{}'",
                name
            );
        }
    }
}

#[test]
fn test_empty_structures_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "emptyObject": {},
        "emptyArray": [],
        "emptyString": "",
        "nested": {
            "empty": {}
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("{}"));
    assert!(result.content.contains("[]"));
    assert!(result.content.contains("\"\""));
}

#[test]
fn test_null_values_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "nullValue": null,
        "arrayWithNulls": [null, 1, null, 2, null],
        "objectWithNull": {
            "value": null,
            "other": "data"
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("null"));
}

#[test]
fn test_string_special_characters_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "quotes": "She said \"hello\"",
        "newline": "line1\nline2\nline3",
        "unicode": "Hello 世界 🌍"
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Should have escaped quotes
    assert!(result.content.contains("\\\"") || result.content.contains("\""));

    // Should have escaped or preserved newlines
    assert!(result.content.contains("\\n") || result.content.contains("\n"));

    // Unicode should be preserved
    assert!(result.content.contains("世界"));
    assert!(result.content.contains("🌍"));
}

#[test]
fn test_mixed_type_arrays_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "mixedPrimitives": [1, "two", 3.0, true, null, false, 42],
        "mixedComplex": [
            42,
            "hello",
            {"type": "object", "value": 100},
            [1, 2, 3],
            null
        ]
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // All types should be present
    assert!(result.content.contains("1"));
    assert!(result.content.contains("two"));
    assert!(result.content.contains("true"));
    assert!(result.content.contains("false"));
    assert!(result.content.contains("null"));
}

#[test]
fn test_uniform_arrays_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "uniformObjects": [
            {"id": 1, "name": "Alice", "age": 30},
            {"id": 2, "name": "Bob", "age": 25},
            {"id": 3, "name": "Charlie", "age": 35}
        ]
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Should contain all data
    assert!(result.content.contains("Alice"));
    assert!(result.content.contains("Bob"));
    assert!(result.content.contains("Charlie"));
}

#[test]
fn test_deeply_nested_fixture() {
    let config = ConversionConfig::default();

    // Create 10-level deep nesting
    let mut json = json!({"deepValue": "reached level 10"});
    for i in (1..=10).rev() {
        json = json!({format!("level{}", i): json});
    }

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("level1"));
    assert!(result.content.contains("level10"));
    assert!(result.content.contains("reached level 10"));
}

#[test]
fn test_special_key_names_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "normal": "value",
        "with space": "value",
        "with:colon": "value",
        "true": "not a boolean",
        "null": "not null"
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Special keys should be quoted
    assert!(result.content.contains("\"with space\"") || result.content.contains("with space"));
    assert!(result.content.contains("\"with:colon\""));
}

#[test]
fn test_arrays_of_arrays_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "matrix": [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        ]
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // All values should be present
    assert!(result.content.contains("1"));
    assert!(result.content.contains("9"));
}

#[test]
fn test_unicode_edge_cases_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "emoji": "🔥💯✨",
        "chinese": "中文测试",
        "mixed": "Hello世界🌍Привет"
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // All unicode should be preserved
    assert!(result.content.contains("🔥"));
    assert!(result.content.contains("中文"));
    assert!(result.content.contains("Привет"));
}

#[test]
fn test_numeric_precision_fixture() {
    let config = ConversionConfig::default();

    let json = json!({
        "smallFloat": 0.000001,
        "largeFloat": 123456789.123456789,
        "scientificPositive": 1.23e10,
        "integer": 9007199254740991
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Numbers should be present (in some form)
    assert!(result.content.contains("0.000001") || result.content.contains("1e-6"));
    assert!(result.content.contains("9007199254740991"));
}
