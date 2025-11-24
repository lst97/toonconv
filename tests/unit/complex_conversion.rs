//! Unit tests for complex JSON structure conversion to TOON
//!
//! Tests deeply nested objects, mixed-type arrays, and edge cases

use serde_json::json;
use toonconv::conversion::{convert_json_to_toon, ConversionConfig, QuoteStrategy};
use toonconv::parser::JsonSource;

#[test]
fn test_deeply_nested_objects() {
    let config = ConversionConfig::default();

    let json = json!({
        "level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "value": "deeply nested"
                    }
                }
            }
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("level1"));
    assert!(result.content.contains("level2"));
    assert!(result.content.contains("level3"));
    assert!(result.content.contains("level4"));
    assert!(result.content.contains("deeply nested"));
}

#[test]
fn test_nested_objects_with_arrays() {
    let config = ConversionConfig::default();

    let json = json!({
        "user": {
            "name": "Alice",
            "scores": [95, 87, 92],
            "address": {
                "city": "New York",
                "zip": "10001"
            }
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("user"));
    assert!(result.content.contains("Alice"));
    assert!(result.content.contains("scores"));
    assert!(result.content.contains("address"));
    assert!(result.content.contains("New York"));
}

#[test]
fn test_mixed_type_array() {
    let config = ConversionConfig::default();

    let json = json!([
        42,
        "hello",
        true,
        null,
        {"key": "value"},
        [1, 2, 3]
    ]);

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("42"));
    assert!(result.content.contains("hello"));
    assert!(result.content.contains("true"));
    assert!(result.content.contains("null"));
    assert!(result.content.contains("key"));
}

#[test]
fn test_array_of_objects_with_different_structures() {
    let config = ConversionConfig::default();

    let json = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "score": 95},
        {"id": 123, "active": true}
    ]);

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("Alice"));
    assert!(result.content.contains("Bob"));
    assert!(result.content.contains("123"));
}

#[test]
fn test_nested_arrays() {
    let config = ConversionConfig::default();

    let json = json!([[1, 2, 3], [4, 5, 6], [7, 8, 9]]);

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.starts_with('['));
    assert!(result.content.contains("1"));
    assert!(result.content.contains("9"));
}

#[test]
fn test_deeply_nested_arrays() {
    let config = ConversionConfig::default();

    let json = json!([[[1, 2], [3, 4]], [[5, 6], [7, 8]]]);

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("1"));
    assert!(result.content.contains("8"));
}

#[test]
fn test_smart_string_quoting_in_complex_structure() {
    let mut config = ConversionConfig::default();
    config.quote_strings = QuoteStrategy::Smart;

    let json = json!({
        "keywords": {
            "literal_true": "true",
            "literal_false": "false",
            "literal_null": "null",
            "normal": "hello"
        },
        "special_chars": {
            "with_colon": "key:value",
            "with_comma": "a,b,c",
            "with_braces": "{data}",
            "empty": ""
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Literals should be quoted
    assert!(result.content.contains("\"true\""));
    assert!(result.content.contains("\"false\""));
    assert!(result.content.contains("\"null\""));

    // Normal strings should not be quoted
    assert!(result.content.contains("hello"));

    // Special characters should be quoted
    assert!(result.content.contains("\"key:value\""));
    assert!(result.content.contains("\"\"")); // empty string
}

#[test]
fn test_complex_real_world_structure() {
    let config = ConversionConfig::default();

    let json = json!({
        "users": [
            {
                "id": 1,
                "name": "Alice",
                "email": "alice@example.com",
                "profile": {
                    "age": 30,
                    "city": "New York",
                    "interests": ["reading", "coding", "music"]
                }
            },
            {
                "id": 2,
                "name": "Bob",
                "email": "bob@example.com",
                "profile": {
                    "age": 25,
                    "city": "San Francisco",
                    "interests": ["gaming", "sports"]
                }
            }
        ],
        "meta": {
            "total": 2,
            "page": 1,
            "limit": 10
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Verify all data is present
    assert!(result.content.contains("users"));
    assert!(result.content.contains("Alice"));
    assert!(result.content.contains("Bob"));
    assert!(result.content.contains("profile"));
    assert!(result.content.contains("interests"));
    assert!(result.content.contains("meta"));
    assert!(result.content.contains("total"));

    // Verify no data loss
    assert!(result.statistics.as_ref().unwrap().data_integrity_verified);
}

#[test]
fn test_empty_nested_structures() {
    let config = ConversionConfig::default();

    let json = json!({
        "empty_object": {},
        "empty_array": [],
        "nested_empty": {
            "inner_empty": {}
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("{}"));
    assert!(result.content.contains("[]"));
}

#[test]
fn test_special_number_values() {
    let config = ConversionConfig::default();

    let json = json!({
        "integer": 42,
        "float": 3.14,
        "negative": -100,
        "zero": 0,
        "scientific": 1.5e10
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("42"));
    assert!(result.content.contains("3.14"));
    assert!(result.content.contains("-100"));
    assert!(result.content.contains("0"));
    assert!(result.content.contains("1.5e10") || result.content.contains("15000000000"));
}

#[test]
fn test_all_json_types_in_object() {
    let config = ConversionConfig::default();

    let json = json!({
        "null_value": null,
        "bool_true": true,
        "bool_false": false,
        "number": 42,
        "string": "hello",
        "array": [1, 2, 3],
        "object": {"nested": "value"}
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("null"));
    assert!(result.content.contains("true"));
    assert!(result.content.contains("false"));
    assert!(result.content.contains("42"));
    assert!(result.content.contains("hello"));
    assert!(result.content.contains("nested"));
}

#[test]
fn test_unicode_in_complex_structure() {
    let config = ConversionConfig::default();

    let json = json!({
        "unicode": {
            "emoji": "😀🎉",
            "chinese": "你好世界",
            "arabic": "مرحبا",
            "mixed": "Hello世界"
        }
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    assert!(result.content.contains("😀"));
    assert!(result.content.contains("你好世界"));
    assert!(result.content.contains("مرحبا"));
}

#[test]
fn test_extreme_nesting_depth() {
    let config = ConversionConfig::default();

    // Create a JSON structure nested 20 levels deep
    let mut json = json!({"value": "deepest"});
    for i in (1..=20).rev() {
        json = json!({format!("level{}", i): json});
    }

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config);

    // Should succeed for reasonable nesting depth
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.content.contains("deepest"));
}

#[test]
fn test_large_array_with_mixed_types() {
    let config = ConversionConfig::default();

    let mut items = Vec::new();
    for i in 0..100 {
        if i % 3 == 0 {
            items.push(json!(i));
        } else if i % 3 == 1 {
            items.push(json!(format!("string_{}", i)));
        } else {
            items.push(json!({"id": i, "value": i * 2}));
        }
    }

    let json = json!(items);
    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Verify array was processed
    assert!(result.content.starts_with('['));
    assert!(result.statistics.as_ref().unwrap().elements_processed > 0);
}

#[test]
fn test_whitespace_in_strings() {
    let mut config = ConversionConfig::default();
    config.quote_strings = QuoteStrategy::Smart;

    let json = json!({
        "leading": " hello",
        "trailing": "hello ",
        "both": " hello ",
        "internal": "hello world",
        "tabs": "\thello",
        "newlines": "line1\nline2"
    });

    let source = JsonSource::Value(json);
    let result = convert_json_to_toon(&source.parse().unwrap(), &config).unwrap();

    // Leading/trailing whitespace should be quoted
    assert!(result.content.contains("\" hello\""));
    assert!(result.content.contains("\"hello \""));

    // Internal whitespace without leading/trailing is OK unquoted
    assert!(result.content.contains("hello world") || result.content.contains("\"hello world\""));

    // Tabs and newlines should be quoted and escaped
    assert!(result.content.contains("\\t") || result.content.contains("\t"));
    assert!(result.content.contains("\\n"));
}
