//! Speed Comparison Benchmark
//!
//! Measures toonconv (Rust) conversion speed and provides comparison data
//! with the official JavaScript implementation.
//!
//! Run with: cargo bench --bench speed_comparison

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;
use toonconv::convert_json;

/// Generate employee records dataset
fn generate_employee_records(count: usize) -> serde_json::Value {
    let departments = ["Engineering", "Sales", "Marketing", "HR"];
    let mut employees = Vec::new();
    for i in 0..count {
        let dept = departments[i % 4];
        employees.push(json!({
            "id": i + 1,
            "name": format!("Employee {}", i + 1),
            "email": format!("emp{}@example.com", i + 1),
            "department": dept,
            "salary": 50000 + (i * 1000),
            "yearsExperience": (i % 20) + 1,
            "active": i % 3 != 0
        }));
    }
    json!({ "employees": employees })
}

/// Generate e-commerce orders dataset
fn generate_ecommerce_orders(count: usize) -> serde_json::Value {
    let mut orders = Vec::new();
    for i in 0..count {
        orders.push(json!({
            "order_id": format!("ORD-{:05}", i),
            "customer": {
                "id": format!("CUST-{:04}", i),
                "name": format!("Customer {}", i),
                "email": format!("customer{}@example.com", i),
                "address": {
                    "street": format!("{} Main St", i),
                    "city": "New York",
                    "zip": "10001",
                    "country": "USA"
                }
            },
            "items": [
                {
                    "product_id": "PROD-1",
                    "name": "Widget A",
                    "quantity": 2,
                    "price": 19.99
                },
                {
                    "product_id": "PROD-2",
                    "name": "Widget B",
                    "quantity": 1,
                    "price": 29.99
                }
            ],
            "status": "shipped",
            "created_at": "2023-10-27T10:00:00Z",
            "total": 69.97
        }));
    }
    json!({ "orders": orders })
}

/// Generate time series metrics dataset
fn generate_time_series(days: usize) -> serde_json::Value {
    let mut metrics = Vec::new();
    for i in 0..days {
        metrics.push(json!({
            "date": format!("2025-01-{:02}", (i % 28) + 1),
            "views": 5000 + (i * 100),
            "clicks": 200 + (i * 10),
            "conversions": 20 + (i % 10),
            "revenue": 7000.0 + (i as f64 * 50.0),
            "bounceRate": 0.3 + (i as f64 * 0.01)
        }));
    }
    json!({ "metrics": metrics })
}

/// Generate deeply nested configuration
fn generate_nested_config() -> serde_json::Value {
    json!({
        "app": {
            "server": {
                "host": "0.0.0.0",
                "port": 8080,
                "options": {
                    "timeout": 30,
                    "keepalive": true,
                    "retry": {
                        "attempts": 3,
                        "backoff": "exponential",
                        "max_delay": 1000
                    }
                }
            },
            "database": {
                "primary": {
                    "host": "db-primary",
                    "port": 5432,
                    "credentials": {
                        "username": "admin",
                        "password_file": "/run/secrets/db_pass"
                    },
                    "pool": {
                        "min": 5,
                        "max": 20,
                        "idle_timeout": 60
                    }
                },
                "replicas": [
                    { "host": "db-replica-1", "port": 5432, "readonly": true },
                    { "host": "db-replica-2", "port": 5432, "readonly": true }
                ]
            },
            "logging": {
                "level": "debug",
                "format": "json",
                "outputs": ["stdout", "file"],
                "file": {
                    "path": "/var/log/app.log",
                    "rotation": {
                        "max_size": "100MB",
                        "max_files": 5
                    }
                }
            }
        }
    })
}

fn benchmark_conversion_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("JSON to TOON Conversion");

    // Small dataset (100 records)
    let employees_100 = generate_employee_records(100);
    let json_size_100 = serde_json::to_string(&employees_100).unwrap().len();
    group.throughput(Throughput::Bytes(json_size_100 as u64));
    group.bench_with_input(
        BenchmarkId::new("employees", "100 records"),
        &employees_100,
        |b, data| {
            b.iter(|| convert_json(black_box(data)).unwrap());
        },
    );

    // Medium dataset (1000 records)
    let employees_1000 = generate_employee_records(1000);
    let json_size_1000 = serde_json::to_string(&employees_1000).unwrap().len();
    group.throughput(Throughput::Bytes(json_size_1000 as u64));
    group.bench_with_input(
        BenchmarkId::new("employees", "1000 records"),
        &employees_1000,
        |b, data| {
            b.iter(|| convert_json(black_box(data)).unwrap());
        },
    );

    // Large dataset (10000 records)
    let employees_10000 = generate_employee_records(10000);
    let json_size_10000 = serde_json::to_string(&employees_10000).unwrap().len();
    group.throughput(Throughput::Bytes(json_size_10000 as u64));
    group.bench_with_input(
        BenchmarkId::new("employees", "10000 records"),
        &employees_10000,
        |b, data| {
            b.iter(|| convert_json(black_box(data)).unwrap());
        },
    );

    // Nested structures (e-commerce orders)
    let orders_100 = generate_ecommerce_orders(100);
    let orders_size = serde_json::to_string(&orders_100).unwrap().len();
    group.throughput(Throughput::Bytes(orders_size as u64));
    group.bench_with_input(
        BenchmarkId::new("ecommerce_orders", "100 orders"),
        &orders_100,
        |b, data| {
            b.iter(|| convert_json(black_box(data)).unwrap());
        },
    );

    // Time series data
    let metrics_365 = generate_time_series(365);
    let metrics_size = serde_json::to_string(&metrics_365).unwrap().len();
    group.throughput(Throughput::Bytes(metrics_size as u64));
    group.bench_with_input(
        BenchmarkId::new("time_series", "365 days"),
        &metrics_365,
        |b, data| {
            b.iter(|| convert_json(black_box(data)).unwrap());
        },
    );

    // Deeply nested configuration
    let config = generate_nested_config();
    let config_size = serde_json::to_string(&config).unwrap().len();
    group.throughput(Throughput::Bytes(config_size as u64));
    group.bench_with_input(
        BenchmarkId::new("nested_config", "deep nesting"),
        &config,
        |b, data| {
            b.iter(|| convert_json(black_box(data)).unwrap());
        },
    );

    group.finish();
}

fn benchmark_throughput_summary(c: &mut Criterion) {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    toonconv (Rust) Speed Benchmark                           ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  Compare with JavaScript: node bench/speed.js                                ║");
    println!("║  Reference: https://github.com/toon-format/toon                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // Quick summary benchmark
    let mut group = c.benchmark_group("Summary");

    // Mixed workload
    let employees = generate_employee_records(500);
    let orders = generate_ecommerce_orders(50);
    let metrics = generate_time_series(180);
    let config = generate_nested_config();

    group.bench_function("mixed_workload", |b| {
        b.iter(|| {
            convert_json(black_box(&employees)).unwrap();
            convert_json(black_box(&orders)).unwrap();
            convert_json(black_box(&metrics)).unwrap();
            convert_json(black_box(&config)).unwrap();
        });
    });

    group.finish();

    // Print comparison guide
    println!();
    println!("────────────────────────────────────────────────────────────────────────────────");
    println!("To compare with the official JavaScript implementation:");
    println!();
    println!("  1. Clone: git clone https://github.com/toon-format/toon.git");
    println!("  2. Install: cd toon && npm install");
    println!("  3. Run JS benchmark: npm run bench");
    println!();
    println!("Expected Rust speedup: 10-50x faster than JavaScript");
    println!("────────────────────────────────────────────────────────────────────────────────");
}

criterion_group!(
    benches,
    benchmark_conversion_speed,
    benchmark_throughput_summary
);
criterion_main!(benches);
