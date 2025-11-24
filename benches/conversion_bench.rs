use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use toonconv::{convert_json, ConversionConfig};

fn benchmark_json_to_toon_conversion(c: &mut Criterion) {
    // Simple object benchmark
    c.bench_function("simple_object", |b| {
        let json = json!({
            "name": "Alice",
            "age": 30,
            "active": true,
            "balance": 1250.50
        });
        b.iter(|| convert_json(black_box(&json)))
    });

    // Array benchmark
    c.bench_function("user_array", |b| {
        let json = json!({
            "users": [
                {"id": 1, "name": "Alice", "role": "admin"},
                {"id": 2, "name": "Bob", "role": "user"},
                {"id": 3, "name": "Charlie", "role": "editor"}
            ]
        });
        b.iter(|| convert_json(black_box(&json)))
    });

    // Nested structure benchmark
    c.bench_function("nested_structure", |b| {
        let json = json!({
            "metadata": {
                "version": 1,
                "author": "system",
                "settings": {
                    "debug": true,
                    "timeout": 30
                }
            },
            "data": {
                "items": [
                    {"id": 1, "name": "Item1", "tags": ["urgent", "pending"]},
                    {"id": 2, "name": "Item2", "tags": ["normal"]}
                ]
            }
        });
        b.iter(|| convert_json(black_box(&json)))
    });

    // Large array benchmark
    c.bench_function("large_array", |b| {
        let mut users = Vec::new();
        for i in 0..1000 {
            users.push(json!({
                "id": i,
                "name": format!("User{}", i),
                "email": format!("user{}@example.com", i),
                "active": i % 2 == 0
            }));
        }
        let json = json!({ "users": users });
        b.iter(|| convert_json(black_box(&json)))
    });

    // Custom configuration benchmark
    c.bench_function("custom_config", |b| {
        let json = json!({
            "name": "Test",
            "data": [1, 2, 3, 4, 5],
            "nested": {"key": "value"}
        });
        let config = ConversionConfig {
            indent_size: 4,
            delimiter: toonconv::conversion::DelimiterType::Tab,
            length_marker: true,
            quote_strings: toonconv::conversion::QuoteStrategy::Smart,
            memory_limit: 1024 * 1024 * 1024, // 1GB
            timeout: std::time::Duration::from_secs(600),
            enable_simd: false,
            pretty: true,
            validate_output: false,
            include_schema: true,
            max_depth: Some(1000),
        };
        b.iter(|| toonconv::convert_json_with_config(black_box(&json), black_box(&config)))
    });
}

fn benchmark_performance_profiles(c: &mut Criterion) {
    let json = json!({
        "complex": {
            "nested": {
                "data": [
                    {"id": 1, "name": "Item1", "value": 1.5},
                    {"id": 2, "name": "Item2", "value": 3.0},
                    {"id": 3, "name": "Item3", "value": 4.5}
                ]
            }
        }
    });

    // Small files profile
    c.bench_function("small_files_profile", |b| {
        let config = ConversionConfig::small_files();
        b.iter(|| toonconv::convert_json_with_config(black_box(&json), black_box(&config)))
    });

    // Large files profile
    c.bench_function("large_files_profile", |b| {
        let config = ConversionConfig::large_files();
        b.iter(|| toonconv::convert_json_with_config(black_box(&json), black_box(&config)))
    });

    // Batch processing profile
    c.bench_function("batch_processing_profile", |b| {
        let config = ConversionConfig::batch_processing();
        b.iter(|| toonconv::convert_json_with_config(black_box(&json), black_box(&config)))
    });
}

criterion_group!(
    benches,
    benchmark_json_to_toon_conversion,
    benchmark_performance_profiles
);
criterion_main!(benches);
