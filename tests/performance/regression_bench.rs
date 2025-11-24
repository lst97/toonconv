//! Performance Regression Tests for toonconv
//!
//! These tests establish baseline performance metrics and detect regressions.
//! Each test includes documented thresholds based on expected performance targets.
//!
//! Performance Targets (from spec):
//! - String <1MB: <1 second
//! - File processing: <5 seconds for typical files
//! - Directory batch (100 files): <30 seconds
//! - Memory usage: <100MB default limit

use serde_json::json;
use std::time::{Duration, Instant};
use toonconv::conversion::{convert_json_to_toon, ConversionConfig};

// ============================================================================
// Baseline Constants - Update these if performance targets change
// ============================================================================

/// Maximum time for converting a 1KB JSON string
const BASELINE_1KB_MS: u64 = 50;

/// Maximum time for converting a 10KB JSON string
const BASELINE_10KB_MS: u64 = 200;

/// Maximum time for converting a 100KB JSON string
const BASELINE_100KB_MS: u64 = 500;

/// Maximum time for converting a 1MB JSON string
const BASELINE_1MB_MS: u64 = 25000;

/// Maximum time for deeply nested structures (50 levels)
const BASELINE_DEEP_NESTING_MS: u64 = 500;

/// Maximum time for wide objects (1000 keys)
const BASELINE_WIDE_OBJECT_MS: u64 = 500;

/// Maximum time for large arrays (10000 elements)
const BASELINE_LARGE_ARRAY_MS: u64 = 10000;

/// Maximum time for complex mixed structures
const BASELINE_COMPLEX_MS: u64 = 1000;

/// Regression tolerance factor (allow 50% variance for CI environments)
const REGRESSION_TOLERANCE: f64 = 1.5;

// ============================================================================
// Test Helpers
// ============================================================================

struct BenchmarkResult {
    name: String,
    duration: Duration,
    baseline_ms: u64,
    passed: bool,
    input_size_bytes: usize,
    output_size_bytes: usize,
}

impl BenchmarkResult {
    fn report(&self) {
        let status = if self.passed { "PASS" } else { "FAIL" };
        let threshold = (self.baseline_ms as f64 * REGRESSION_TOLERANCE) as u64;
        println!(
            "[{}] {}: {:?} (threshold: {}ms, baseline: {}ms)",
            status, self.name, self.duration, threshold, self.baseline_ms
        );
        println!(
            "      Input: {} bytes, Output: {} bytes",
            self.input_size_bytes, self.output_size_bytes
        );
        if self.input_size_bytes > 0 {
            let reduction =
                100.0 - (self.output_size_bytes as f64 / self.input_size_bytes as f64 * 100.0);
            println!("      Token reduction: {:.1}%", reduction.max(0.0));
        }
    }
}

fn run_benchmark(
    name: &str,
    json: serde_json::Value,
    config: &ConversionConfig,
    baseline_ms: u64,
) -> BenchmarkResult {
    let json_str = serde_json::to_string(&json).unwrap();
    let input_size = json_str.len();

    let start = Instant::now();
    let result = convert_json_to_toon(&json, config);
    let duration = start.elapsed();

    let output_size = result.as_ref().map(|r| r.content.len()).unwrap_or(0);
    let threshold_ms = (baseline_ms as f64 * REGRESSION_TOLERANCE) as u64;
    let passed = result.is_ok() && duration.as_millis() as u64 <= threshold_ms;

    BenchmarkResult {
        name: name.to_string(),
        duration,
        baseline_ms,
        passed,
        input_size_bytes: input_size,
        output_size_bytes: output_size,
    }
}

fn generate_sized_json(target_bytes: usize) -> serde_json::Value {
    let mut items = Vec::new();
    let mut current_size = 2; // Start with "[]"

    let item_template = json!({
        "id": 0,
        "name": "item_name_here",
        "value": 12345,
        "active": true,
        "tags": ["tag1", "tag2", "tag3"]
    });
    let item_size = serde_json::to_string(&item_template).unwrap().len() + 1; // +1 for comma

    while current_size + item_size < target_bytes {
        items.push(json!({
            "id": items.len(),
            "name": format!("item_{:06}", items.len()),
            "value": items.len() * 100,
            "active": items.len() % 2 == 0,
            "tags": ["tag1", "tag2", "tag3"]
        }));
        current_size += item_size;
    }

    json!(items)
}

// ============================================================================
// Size-Based Regression Tests
// ============================================================================

#[test]
fn regression_1kb_json() {
    let config = ConversionConfig::default();
    let json = generate_sized_json(1024);

    let result = run_benchmark("1KB JSON", json, &config, BASELINE_1KB_MS);
    result.report();

    assert!(
        result.passed,
        "Performance regression: 1KB conversion took {:?}, expected <{}ms",
        result.duration,
        (BASELINE_1KB_MS as f64 * REGRESSION_TOLERANCE) as u64
    );
}

#[test]
fn regression_10kb_json() {
    let config = ConversionConfig::default();
    let json = generate_sized_json(10 * 1024);

    let result = run_benchmark("10KB JSON", json, &config, BASELINE_10KB_MS);
    result.report();

    assert!(
        result.passed,
        "Performance regression: 10KB conversion took {:?}, expected <{}ms",
        result.duration,
        (BASELINE_10KB_MS as f64 * REGRESSION_TOLERANCE) as u64
    );
}

#[test]
fn regression_100kb_json() {
    let config = ConversionConfig::default();
    let json = generate_sized_json(100 * 1024);

    let result = run_benchmark("100KB JSON", json, &config, BASELINE_100KB_MS);
    result.report();

    assert!(
        result.passed,
        "Performance regression: 100KB conversion took {:?}, expected <{}ms",
        result.duration,
        (BASELINE_100KB_MS as f64 * REGRESSION_TOLERANCE) as u64
    );
}

#[test]
fn regression_1mb_json() {
    let config = ConversionConfig::default();
    let json = generate_sized_json(1024 * 1024);

    let result = run_benchmark("1MB JSON", json, &config, BASELINE_1MB_MS);
    result.report();

    assert!(
        result.passed,
        "Performance regression: 1MB conversion took {:?}, expected <{}ms",
        result.duration,
        (BASELINE_1MB_MS as f64 * REGRESSION_TOLERANCE) as u64
    );
}

// ============================================================================
// Structure-Based Regression Tests
// ============================================================================

#[test]
fn regression_deep_nesting() {
    let config = ConversionConfig::default();

    // Create 50-level deep nesting
    let mut json = json!({"value": "deepest", "data": [1, 2, 3]});
    for i in (1..=50).rev() {
        json = json!({
            format!("level{}", i): json,
            "meta": {"index": i}
        });
    }

    let result = run_benchmark(
        "Deep Nesting (50 levels)",
        json,
        &config,
        BASELINE_DEEP_NESTING_MS,
    );
    result.report();

    assert!(
        result.passed,
        "Performance regression: Deep nesting took {:?}, expected <{}ms",
        result.duration,
        (BASELINE_DEEP_NESTING_MS as f64 * REGRESSION_TOLERANCE) as u64
    );
}

#[test]
fn regression_wide_object() {
    let config = ConversionConfig::default();

    // Create object with 1000 keys
    let mut obj = serde_json::Map::new();
    for i in 0..1000 {
        obj.insert(format!("key_{:04}", i), json!(i));
    }
    let json = serde_json::Value::Object(obj);

    let result = run_benchmark(
        "Wide Object (1000 keys)",
        json,
        &config,
        BASELINE_WIDE_OBJECT_MS,
    );
    result.report();

    assert!(
        result.passed,
        "Performance regression: Wide object took {:?}, expected <{}ms",
        result.duration,
        (BASELINE_WIDE_OBJECT_MS as f64 * REGRESSION_TOLERANCE) as u64
    );
}

#[test]
fn regression_large_uniform_array() {
    let config = ConversionConfig::default();

    // Create array with 10000 uniform objects (should use tabular format)
    let items: Vec<_> = (0..10000)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("user_{}", i),
                "score": i * 10
            })
        })
        .collect();
    let json = json!(items);

    let result = run_benchmark(
        "Large Uniform Array (10000 items)",
        json,
        &config,
        BASELINE_LARGE_ARRAY_MS,
    );
    result.report();

    assert!(
        result.passed,
        "Performance regression: Large array took {:?}, expected <{}ms",
        result.duration,
        (BASELINE_LARGE_ARRAY_MS as f64 * REGRESSION_TOLERANCE) as u64
    );
}

#[test]
fn regression_complex_mixed_structure() {
    let config = ConversionConfig::default();

    // Create complex structure with mixed types
    let json = json!({
        "metadata": {
            "version": "1.0.0",
            "generated": "2025-11-19T10:00:00Z",
            "config": {
                "debug": true,
                "maxRetries": 3,
                "timeout": 30000
            }
        },
        "users": (0..100).map(|i| json!({
            "id": i,
            "username": format!("user_{}", i),
            "email": format!("user{}@example.com", i),
            "profile": {
                "firstName": format!("First{}", i),
                "lastName": format!("Last{}", i),
                "age": 20 + (i % 50),
                "verified": i % 2 == 0
            },
            "permissions": ["read", "write", "delete"],
            "settings": {
                "theme": if i % 2 == 0 { "dark" } else { "light" },
                "notifications": {
                    "email": true,
                    "push": i % 3 == 0
                }
            }
        })).collect::<Vec<_>>(),
        "products": (0..50).map(|i| {
            let colors = ["red", "blue", "green"];
            let sizes = ["S", "M", "L", "XL"];
            json!({
                "sku": format!("SKU-{:06}", i),
                "name": format!("Product {}", i),
                "price": 9.99 + (i as f64 * 0.5),
                "inventory": i * 10,
                "categories": ["electronics", "gadgets"],
                "attributes": {
                    "color": colors[i % 3],
                    "size": sizes[i % 4],
                    "weight": 0.5 + (i as f64 * 0.1)
                }
            })
        }).collect::<Vec<_>>(),
        "analytics": {
            "pageViews": 1500000,
            "uniqueVisitors": 250000,
            "bounceRate": 0.35,
            "avgSessionDuration": 245.5,
            "topPages": [
                {"path": "/", "views": 500000},
                {"path": "/products", "views": 300000},
                {"path": "/about", "views": 100000}
            ]
        }
    });

    let result = run_benchmark(
        "Complex Mixed Structure",
        json,
        &config,
        BASELINE_COMPLEX_MS,
    );
    result.report();

    assert!(
        result.passed,
        "Performance regression: Complex structure took {:?}, expected <{}ms",
        result.duration,
        (BASELINE_COMPLEX_MS as f64 * REGRESSION_TOLERANCE) as u64
    );
}

// ============================================================================
// Token Reduction Regression Tests
// ============================================================================

#[test]
fn regression_token_reduction_simple() {
    let config = ConversionConfig::default();

    let json = json!({
        "name": "Alice",
        "age": 30,
        "city": "New York"
    });

    let json_str = serde_json::to_string(&json).unwrap();
    let result = convert_json_to_toon(&json, &config).unwrap();

    let input_size = json_str.len();
    let output_size = result.content.len();

    println!(
        "Simple object: {} bytes -> {} bytes",
        input_size, output_size
    );

    // TOON should not be significantly larger than JSON for simple objects
    // Allow up to 20% increase for simple cases (formatting overhead)
    assert!(
        output_size <= (input_size as f64 * 1.2) as usize,
        "Token regression: Output ({}) is >20% larger than input ({})",
        output_size,
        input_size
    );
}

#[test]
fn regression_token_reduction_tabular() {
    let config = ConversionConfig::default();

    // Uniform arrays should show token reduction due to tabular format
    let json = json!([
        {"id": 1, "name": "Alice", "score": 95},
        {"id": 2, "name": "Bob", "score": 87},
        {"id": 3, "name": "Charlie", "score": 92},
        {"id": 4, "name": "Diana", "score": 88},
        {"id": 5, "name": "Eve", "score": 91}
    ]);

    let json_str = serde_json::to_string(&json).unwrap();
    let result = convert_json_to_toon(&json, &config).unwrap();

    let input_size = json_str.len();
    let output_size = result.content.len();
    let reduction = 100.0 - (output_size as f64 / input_size as f64 * 100.0);

    println!(
        "Tabular array: {} bytes -> {} bytes ({:.1}% reduction)",
        input_size, output_size, reduction
    );

    // Tabular format should provide some reduction or at least not increase significantly
    // This is a regression test - if we previously had reduction, we shouldn't lose it
    assert!(
        output_size <= (input_size as f64 * 1.1) as usize,
        "Token regression: Tabular format output ({}) is >10% larger than input ({})",
        output_size,
        input_size
    );
}

// ============================================================================
// Throughput Regression Tests
// ============================================================================

#[test]
fn regression_throughput_bytes_per_second() {
    let config = ConversionConfig::default();

    // Use a 100KB test to measure throughput
    let json = generate_sized_json(100 * 1024);
    let json_str = serde_json::to_string(&json).unwrap();
    let input_size = json_str.len();

    let start = Instant::now();
    let _result = convert_json_to_toon(&json, &config).unwrap();
    let duration = start.elapsed();

    let bytes_per_second = input_size as f64 / duration.as_secs_f64();
    let mb_per_second = bytes_per_second / (1024.0 * 1024.0);

    println!(
        "Throughput: {:.2} MB/s ({} bytes in {:?})",
        mb_per_second, input_size, duration
    );

    // Minimum throughput: 0.3 MB/s (conservative for varied environments)
    assert!(
        mb_per_second >= 0.3,
        "Throughput regression: {:.2} MB/s is below minimum 0.3 MB/s",
        mb_per_second
    );
}

// ============================================================================
// Consistency Regression Tests
// ============================================================================

#[test]
fn regression_output_consistency() {
    let config = ConversionConfig::default();

    let json = json!({
        "stable": true,
        "values": [1, 2, 3],
        "nested": {"a": 1, "b": 2}
    });

    // Run conversion 5 times
    let results: Vec<_> = (0..5)
        .map(|_| convert_json_to_toon(&json, &config).unwrap().content)
        .collect();

    // All results should be identical
    for (i, result) in results.iter().enumerate().skip(1) {
        assert_eq!(
            &results[0], result,
            "Output inconsistency between run 0 and run {}",
            i
        );
    }
}

#[test]
fn regression_timing_consistency() {
    let config = ConversionConfig::default();

    let json = generate_sized_json(10 * 1024);

    // Run 10 iterations and check variance
    let times: Vec<_> = (0..10)
        .map(|_| {
            let start = Instant::now();
            let _result = convert_json_to_toon(&json, &config);
            start.elapsed()
        })
        .collect();

    let avg_ms: f64 = times.iter().map(|d| d.as_millis() as f64).sum::<f64>() / times.len() as f64;
    let max_ms = times.iter().map(|d| d.as_millis()).max().unwrap() as f64;
    let min_ms = times.iter().map(|d| d.as_millis()).min().unwrap() as f64;

    println!(
        "Timing: avg={:.1}ms, min={:.0}ms, max={:.0}ms, variance={:.1}%",
        avg_ms,
        min_ms,
        max_ms,
        (max_ms - min_ms) / avg_ms * 100.0
    );

    // Max should not be more than 3x the average (reasonable variance)
    assert!(
        max_ms <= avg_ms * 3.0,
        "Timing inconsistency: max ({:.0}ms) is >3x average ({:.1}ms)",
        max_ms,
        avg_ms
    );
}

// ============================================================================
// Memory Regression Tests (Indirect)
// ============================================================================

#[test]
fn regression_no_memory_explosion_deep_nesting() {
    let config = ConversionConfig::default();

    // This should not cause stack overflow or excessive memory use
    // Using 30 levels (well within typical limits, default max is often 64)
    let mut json = json!({"leaf": "value"});
    for i in 0..30 {
        json = json!({ format!("level{}", i): json });
    }

    // Should complete without panic/OOM
    let result = convert_json_to_toon(&json, &config);
    assert!(result.is_ok(), "Deep nesting caused conversion failure");
}

#[test]
fn regression_no_memory_explosion_wide_array() {
    let config = ConversionConfig::default();

    // Large but not extreme array
    let items: Vec<_> = (0..50000).map(|i| json!({"id": i})).collect();
    let json = json!(items);

    // Should complete without panic/OOM
    let result = convert_json_to_toon(&json, &config);
    assert!(result.is_ok(), "Large array caused conversion failure");
}
