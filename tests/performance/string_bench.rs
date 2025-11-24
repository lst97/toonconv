//! Performance benchmarks for string-to-TOON conversion
//! 
//! These benchmarks test the performance requirements:
//! - Convert JSON strings to TOON in under 1 second for strings <1MB
//! - Performance degradation with increasing input size
//! - Memory usage under load
//! - Throughput measurements

use toonconv::conversion::{convert_json_string, ConversionConfig};
use toonconv::error::ConversionError;
use std::time::{Duration, Instant};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn generate_test_json(size_target: usize) -> String {
    let mut json_parts = Vec::new();
    let mut current_size = 0;
    
    // Generate user objects
    for i in 0.. {
        let user_obj = format!(r#"{{"id": {}, "name": "user{}", "email": "user{}@example.com", "active": true, "score": {:.2}}}"#,
                              i, i, i, i as f64 * 1.5);
        let obj_with_comma = if i > 0 { "," } else { "" }.to_string() + &user_obj;
        
        if current_size + obj_with_comma.len() > size_target {
            break;
        }
        
        json_parts.push(obj_with_comma);
        current_size += obj_with_comma.len();
    }
    
    format!("[{}]", json_parts.join(""))
}

fn generate_large_nested_json(depth: usize, width: usize) -> String {
    fn build_nested(depth: usize, width: usize) -> String {
        if depth == 0 {
            return format!(r#"{{"value": {}, "flag": true}}"#, depth * 100);
        }
        
        let mut items = Vec::new();
        for i in 0..width {
            items.push(format!(r#"{{"key{}": {}, "nested": {}}}"#, i, i * depth, build_nested(depth - 1, width)));
        }
        
        format!(r#"{{"depth": {}, "items": [{}]}}"#, depth, items.join(","))
    }
    
    build_nested(depth, width)
}

fn benchmark_simple_object_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_object_conversion");
    
    let test_cases = vec![
        ("tiny", r#"{"name": "Alice", "age": 30}"#),
        ("small", r#"{"id": 1, "name": "Alice", "email": "alice@example.com", "active": true, "score": 95.5}"#),
        ("medium", r#"{"user": {"id": 1, "name": "Alice"}, "metadata": {"created": "2023-01-01", "version": 2}, "settings": {"theme": "dark", "notifications": true}}"#),
    ];
    
    for (name, json) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("convert_string", name),
            json,
            |b, json| {
                let config = ConversionConfig::default();
                b.iter(|| {
                    convert_json_string(black_box(json), black_box(&config))
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_array_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_conversion");
    
    // Generate arrays of different sizes
    let sizes = vec![10, 100, 1000, 5000];
    
    for size in sizes {
        let json = format!(r#"[{}]"#, (0..size).map(|i| format!(r#"{{"id": {}, "name": "user{}"}}"#, i, i)).collect::<Vec<_>>().join(","));
        
        group.bench_with_input(
            BenchmarkId::new("uniform_object_array", size),
            &json,
            |b, json| {
                let config = ConversionConfig::default();
                b.iter(|| {
                    convert_json_string(black_box(json), black_box(&config))
                })
            },
        );
        
        // Primitive array
        let primitive_json = format!(r#"[{}]"#, (0..size).collect::<Vec<_>>().iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","));
        
        group.bench_with_input(
            BenchmarkId::new("primitive_array", size),
            &primitive_json,
            |b, json| {
                let config = ConversionConfig::default();
                b.iter(|| {
                    convert_json_string(black_box(json), black_box(&config))
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_performance_requirements(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_requirements");
    
    // Test the main requirement: < 1 second for strings < 1MB
    let json_sizes = vec![1024, 10_240, 102_400, 500_000]; // 1KB, 10KB, 100KB, 500KB
    
    for size in json_sizes {
        let json = generate_test_json(size);
        assert!(json.len() < 1_000_000, "Generated JSON should be under 1MB");
        
        group.bench_with_input(
            BenchmarkId::new("under_1mb_requirement", size),
            &json,
            |b, json| {
                let config = ConversionConfig::default();
                b.iter(|| {
                    let start = Instant::now();
                    let result = convert_json_string(black_box(json), black_box(&config));
                    let elapsed = start.elapsed();
                    
                    // Ensure the operation completed successfully
                    assert!(result.is_ok(), "Conversion should succeed");
                    
                    // The benchmark should verify it's under 1 second
                    assert!(elapsed.as_secs_f32() < 1.0, 
                           "Conversion should be under 1 second, took {:?}", elapsed);
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_complex_nested_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_nested_structures");
    
    let test_cases = vec![
        ("shallow_wide", generate_large_nested_json(2, 50)),
        ("medium_nested", generate_large_nested_json(3, 10)),
        ("deep_narrow", generate_large_nested_json(5, 3)),
    ];
    
    for (name, json) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("nested_conversion", name),
            &json,
            |b, json| {
                let config = ConversionConfig::default();
                b.iter(|| {
                    convert_json_string(black_box(json), black_box(&config))
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_configuration_variations(c: &mut Criterion) {
    let json = r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#;
    let mut group = c.benchmark_group("configuration_variations");
    
    // Test different configurations
    let configs = vec![
        ("default", ConversionConfig::default()),
        ("compact", ConversionConfig::default().with_pretty(false)),
        ("tab_delimiter", ConversionConfig::default().with_delimiter(crate::conversion::DelimiterType::Tab)),
        ("length_markers", ConversionConfig::default().with_length_marker(true)),
    ];
    
    for (name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("config_variation", name),
            &json,
            |b, json| {
                b.iter(|| {
                    convert_json_string(black_box(json), black_box(&config))
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_throughput_measurements(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_measurements");
    
    // Measure conversions per second
    let json = r#"{"name": "Alice", "age": 30, "active": true, "email": "alice@example.com"}"#;
    let iterations = 1000;
    
    group.bench_function("conversions_per_second", |b| {
        let config = ConversionConfig::default();
        b.iter(|| {
            for _ in 0..iterations {
                let _ = convert_json_string(black_box(json), black_box(&config));
            }
        })
    });
    
    group.finish();
}

fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    // Test with different memory limits
    let json_sizes = vec![100_000, 500_000, 900_000]; // 100KB, 500KB, 900KB
    
    for size in json_sizes {
        let json = generate_test_json(size);
        
        group.bench_with_input(
            BenchmarkId::new("memory_limited", size),
            &json,
            |b, json| {
                let mut config = ConversionConfig::default();
                config.memory_limit = size * 2; // Allow 2x the input size
                
                b.iter(|| {
                    let result = convert_json_string(black_box(json), black_box(&config));
                    assert!(result.is_ok(), "Conversion should succeed with adequate memory limit");
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_error_handling_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling_performance");
    
    let invalid_cases = vec![
        ("unterminated_string", r#"{"name": "test"#),
        ("missing_comma", r#"{"name": "test", "age": 30}"#),
        ("trailing_comma", r#"{"name": "test", "age": 30,}"#),
        ("invalid_syntax", r#"{"name": "test", "age": }"#),
    ];
    
    for (name, invalid_json) in invalid_cases {
        group.bench_with_input(
            BenchmarkId::new("invalid_json_error", name),
            invalid_json,
            |b, json| {
                let config = ConversionConfig::default();
                b.iter(|| {
                    let result = convert_json_string(black_box(json), black_box(&config));
                    assert!(result.is_err(), "Invalid JSON should produce error");
                })
            },
        );
    }
    
    group.finish();
}

// Regression test to ensure performance doesn't degrade
fn benchmark_regression_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_tests");
    
    // Known good performance baseline
    let baseline_json = r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#;
    
    group.bench_function("baseline_performance", |b| {
        let config = ConversionConfig::default();
        b.iter(|| {
            let start = Instant::now();
            let result = convert_json_string(black_box(baseline_json), black_box(&config));
            let elapsed = start.elapsed();
            
            assert!(result.is_ok(), "Baseline conversion should succeed");
            assert!(elapsed.as_millis() < 50, 
                   "Baseline conversion should be fast (under 50ms), took {:?}", elapsed);
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_simple_object_conversion,
    benchmark_array_conversion,
    benchmark_performance_requirements,
    benchmark_complex_nested_structures,
    benchmark_configuration_variations,
    benchmark_throughput_measurements,
    benchmark_memory_efficiency,
    benchmark_error_handling_performance,
    benchmark_regression_tests
);
criterion_main!(benches);

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Duration;

    /// Test that basic conversion meets performance requirements
    #[test]
    fn test_basic_performance_requirement() {
        let config = ConversionConfig::default();
        let json = r#"{"name": "Alice", "age": 30}"#;
        
        let start = Instant::now();
        let result = convert_json_string(json, &config);
        let elapsed = start.elapsed();
        
        assert!(result.is_ok(), "Conversion should succeed");
        assert!(elapsed < Duration::from_millis(10), 
               "Basic conversion should be very fast (under 10ms), took {:?}", elapsed);
    }

    /// Test that large JSON meets the < 1 second requirement
    #[test]
    fn test_large_json_performance_requirement() {
        let config = ConversionConfig::default();
        let large_json = generate_test_json(500_000); // 500KB
        
        assert!(large_json.len() < 1_000_000, "Test JSON should be under 1MB");
        
        let start = Instant::now();
        let result = convert_json_string(&large_json, &config);
        let elapsed = start.elapsed();
        
        assert!(result.is_ok(), "Large JSON conversion should succeed");
        assert!(elapsed < Duration::from_secs(1), 
               "Large JSON should convert in under 1 second, took {:?}", elapsed);
    }

    /// Test throughput for multiple conversions
    #[test]
    fn test_conversion_throughput() {
        let config = ConversionConfig::default();
        let json = r#"{"data": "test", "timestamp": 1234567890}"#;
        
        let iterations = 100;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let result = convert_json_string(json, &config);
            assert!(result.is_ok(), "All conversions should succeed");
        }
        
        let total_elapsed = start.elapsed();
        let per_conversion = total_elapsed / iterations;
        
        // Should be able to do at least 100 conversions per second
        assert!(per_conversion < Duration::from_millis(10), 
               "Should handle high throughput (100+ conversions/sec), took {:?} per conversion", 
               per_conversion);
    }

    /// Test performance with memory constraints
    #[test]
    fn test_memory_constrained_performance() {
        let mut config = ConversionConfig::default();
        config.memory_limit = 1024 * 1024; // 1MB limit
        
        let json = generate_test_json(100_000); // 100KB JSON
        
        let start = Instant::now();
        let result = convert_json_string(&json, &config);
        let elapsed = start.elapsed();
        
        assert!(result.is_ok(), "Conversion should succeed with memory limit");
        assert!(elapsed < Duration::from_millis(500), 
               "Conversion with memory limit should still be fast, took {:?}", elapsed);
    }
}