//! Performance benchmarks for complex JSON structures
//!
//! Tests conversion performance for deeply nested objects,
//! large arrays, and mixed-type structures.

use serde_json::json;
use std::time::Instant;
use toonconv::conversion::{convert_json_to_toon, ConversionConfig};
use toonconv::parser::JsonSource;

#[test]
fn test_deeply_nested_object_performance() {
    let config = ConversionConfig::default();

    // Create a deeply nested structure (50 levels)
    let mut json = json!({"value": "deepest", "data": [1, 2, 3]});
    for i in (1..=50).rev() {
        json = json!({
            format!("level{}", i): json,
            "metadata": {"index": i, "active": true}
        });
    }

    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    println!("Deeply nested (50 levels) conversion time: {:?}", duration);

    // Should complete in reasonable time (< 100ms for 50 levels)
    assert!(
        duration.as_millis() < 100,
        "Conversion took too long: {:?}",
        duration
    );
}

#[test]
fn test_large_array_performance() {
    let config = ConversionConfig::default();

    // Create an array with 10,000 elements
    let items: Vec<_> = (0..10000)
        .map(|i| {
            json!({
                "id": i,
                "value": i * 2,
                "label": format!("item_{}", i)
            })
        })
        .collect();

    let json = json!(items);
    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    println!(
        "Large array (10,000 objects) conversion time: {:?}",
        duration
    );

    // Should complete in reasonable time (< 500ms for 10k items)
    assert!(
        duration.as_millis() < 500,
        "Conversion took too long: {:?}",
        duration
    );
}

#[test]
fn test_wide_object_performance() {
    let config = ConversionConfig::default();

    // Create an object with 1,000 fields
    let mut obj = serde_json::Map::new();
    for i in 0..1000 {
        obj.insert(format!("field_{}", i), json!(format!("value_{}", i)));
    }

    let json = json!(obj);
    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    println!("Wide object (1,000 fields) conversion time: {:?}", duration);

    // Should complete quickly (< 50ms for 1k fields)
    assert!(
        duration.as_millis() < 50,
        "Conversion took too long: {:?}",
        duration
    );
}

#[test]
fn test_mixed_complex_structure_performance() {
    let config = ConversionConfig::default();

    // Create a complex real-world-like structure
    let mut users = Vec::new();
    for i in 0..1000 {
        users.push(json!({
            "id": i,
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "profile": {
                "age": 20 + (i % 50),
                "city": format!("City {}", i % 100),
                "interests": vec![
                    format!("hobby_{}", i % 10),
                    format!("hobby_{}", (i + 1) % 10),
                    format!("hobby_{}", (i + 2) % 10)
                ],
                "settings": {
                    "notifications": true,
                    "privacy": "public",
                    "theme": if i % 2 == 0 { "dark" } else { "light" }
                }
            },
            "posts": (0..5).map(|j| json!({
                "id": i * 100 + j,
                "title": format!("Post {} by User {}", j, i),
                "likes": (i + j) % 100
            })).collect::<Vec<_>>()
        }));
    }

    let json = json!({
        "users": users,
        "meta": {
            "total": 1000,
            "page": 1,
            "timestamp": "2025-11-19T00:00:00Z"
        }
    });

    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    println!(
        "Complex mixed structure (1,000 users with nested data) conversion time: {:?}",
        duration
    );

    // Should complete in reasonable time (< 1 second for complex structure)
    assert!(
        duration.as_secs() < 1,
        "Conversion took too long: {:?}",
        duration
    );
}

#[test]
fn test_nested_arrays_performance() {
    let config = ConversionConfig::default();

    // Create a structure with nested arrays (matrix-like)
    let mut matrix = Vec::new();
    for i in 0..100 {
        let mut row = Vec::new();
        for j in 0..100 {
            row.push(json!(i * 100 + j));
        }
        matrix.push(json!(row));
    }

    let json = json!(matrix);
    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    println!(
        "Nested arrays (100x100 matrix) conversion time: {:?}",
        duration
    );

    // Should complete quickly (< 100ms)
    assert!(
        duration.as_millis() < 100,
        "Conversion took too long: {:?}",
        duration
    );
}

#[test]
fn test_string_heavy_structure_performance() {
    let config = ConversionConfig::default();

    // Create a structure with many long strings
    let mut items = Vec::new();
    for i in 0..1000 {
        items.push(json!({
            "id": i,
            "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(10),
            "content": "This is a long content field with lots of text data. ".repeat(20),
            "tags": vec!["tag1", "tag2", "tag3", "tag4", "tag5"]
        }));
    }

    let json = json!(items);
    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    println!(
        "String-heavy structure (1,000 items with long strings) conversion time: {:?}",
        duration
    );

    // Should complete in reasonable time (< 500ms)
    assert!(
        duration.as_millis() < 500,
        "Conversion took too long: {:?}",
        duration
    );
}

#[test]
fn test_uniform_array_tabular_performance() {
    let config = ConversionConfig::default();

    // Create a large uniform array (should use tabular format)
    let mut users = Vec::new();
    for i in 0..5000 {
        users.push(json!({
            "id": i,
            "name": format!("User{}", i),
            "age": 20 + (i % 60),
            "score": (i * 17) % 100
        }));
    }

    let json = json!(users);
    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    let output = result.unwrap();

    println!(
        "Uniform array tabular (5,000 rows) conversion time: {:?}",
        duration
    );

    // Tabular format should be efficient (< 300ms)
    assert!(
        duration.as_millis() < 300,
        "Conversion took too long: {:?}",
        duration
    );

    // Verify tabular format was used (should have schema declaration)
    assert!(output.content.contains("[5000"));
}

#[test]
fn test_memory_efficiency_large_structure() {
    let config = ConversionConfig::default();

    // Create a moderately large structure
    let items: Vec<_> = (0..5000)
        .map(|i| {
            json!({
                "id": i,
                "data": {
                    "values": vec![i, i+1, i+2, i+3, i+4],
                    "metadata": {"index": i, "active": true}
                }
            })
        })
        .collect();

    let json = json!(items);
    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    let output = result.unwrap();

    println!(
        "Large structure (5,000 nested objects) conversion time: {:?}",
        duration
    );
    println!("Output size: {} bytes", output.content.len());

    // Should complete efficiently (< 400ms)
    assert!(
        duration.as_millis() < 400,
        "Conversion took too long: {:?}",
        duration
    );

    // Verify statistics were tracked
    if let Some(stats) = output.statistics {
        assert!(stats.elements_processed > 0);
        println!("Elements processed: {}", stats.elements_processed);
    }
}

#[test]
fn test_pathological_nesting_performance() {
    let config = ConversionConfig::default();

    // Create a structure that alternates between objects and arrays
    let mut json = json!([1, 2, 3]);
    for i in 0..30 {
        if i % 2 == 0 {
            json = json!({"data": json, "level": i});
        } else {
            json = json!([json, {"index": i}]);
        }
    }

    let source = JsonSource::Value(json);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    let result = convert_json_to_toon(&parsed, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    println!(
        "Pathological nesting (30 levels, alternating types) conversion time: {:?}",
        duration
    );

    // Should handle complex nesting (< 50ms)
    assert!(
        duration.as_millis() < 50,
        "Conversion took too long: {:?}",
        duration
    );
}

#[test]
fn test_comparison_simple_vs_complex() {
    let config = ConversionConfig::default();

    // Simple structure
    let simple = json!({"name": "Alice", "age": 30});
    let source = JsonSource::Value(simple);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    convert_json_to_toon(&parsed, &config).unwrap();
    let simple_time = start.elapsed();

    // Complex structure
    let complex = json!({
        "users": (0..100).map(|i| json!({
            "id": i,
            "profile": {
                "data": vec![1, 2, 3, 4, 5],
                "nested": {"level": i}
            }
        })).collect::<Vec<_>>()
    });

    let source = JsonSource::Value(complex);
    let parsed = source.parse().unwrap();

    let start = Instant::now();
    convert_json_to_toon(&parsed, &config).unwrap();
    let complex_time = start.elapsed();

    println!("Simple structure time: {:?}", simple_time);
    println!("Complex structure time: {:?}", complex_time);
    println!(
        "Complexity ratio: {:.2}x",
        complex_time.as_nanos() as f64 / simple_time.as_nanos() as f64
    );

    // Complex should be slower but not exponentially (< 100x)
    let ratio = complex_time.as_nanos() as f64 / simple_time.as_nanos() as f64;
    assert!(
        ratio < 100.0,
        "Performance degradation too severe: {:.2}x",
        ratio
    );
}
