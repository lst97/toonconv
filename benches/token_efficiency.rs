//! Token Efficiency Benchmark
//!
//! Compares token counts across TOON, JSON, JSON compact, YAML, and XML formats.
//! Output format matches the official toon-format/toon repository benchmarks.

use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::json;
use tiktoken_rs::cl100k_base;
use toonconv::convert_json;

/// Count tokens using cl100k_base tokenizer (GPT-4 compatible)
fn count_tokens(text: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    bpe.encode_with_special_tokens(text).len()
}

/// Result structure for a single dataset benchmark
struct BenchmarkResult {
    name: String,
    toon_tokens: usize,
    json_tokens: usize,
    json_compact_tokens: usize,
    yaml_tokens: usize,
    xml_tokens: usize,
    _csv_tokens: Option<usize>,
    tabular_percentage: f64,
}

// === Data Generators (matching official benchmark datasets) ===

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

fn generate_github_repos(count: usize) -> serde_json::Value {
    let mut repos = Vec::new();
    for i in 0..count {
        repos.push(json!({
            "id": 10000000 + i,
            "name": format!("repo-{}", i),
            "repo": format!("org-{}/repo-{}", i % 10, i),
            "description": format!("A sample repository number {}", i),
            "createdAt": "2024-01-15T10:00:00Z",
            "updatedAt": "2025-01-15T10:00:00Z",
            "stars": 1000 + (i * 100),
            "watchers": 100 + (i * 10),
            "forks": 50 + (i * 5),
            "defaultBranch": "main"
        }));
    }
    json!({ "repositories": repos })
}

fn generate_event_logs(count: usize) -> serde_json::Value {
    let mut logs = Vec::new();
    for i in 0..count {
        if i % 10 == 0 {
            // Error log with nested metadata
            logs.push(json!({
                "timestamp": "2023-10-27T10:00:00Z",
                "level": "ERROR",
                "service": "api-gateway",
                "trace_id": format!("trace-{:x}", i * 12345),
                "message": format!("Error processing request {}", i),
                "error": {
                    "code": 500,
                    "type": "InternalError",
                    "stack": "at processRequest (api.js:42)"
                }
            }));
        } else {
            // Info log (flat structure)
            logs.push(json!({
                "timestamp": "2023-10-27T10:00:00Z",
                "level": "INFO",
                "service": "api-gateway",
                "trace_id": format!("trace-{:x}", i * 12345),
                "message": format!("Request processed in {}ms", 10 + (i % 50))
            }));
        }
    }
    json!({ "logs": logs })
}

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

/// Convert JSON to XML (simple implementation for benchmarking)
fn to_xml(val: &serde_json::Value, key: &str) -> String {
    match val {
        serde_json::Value::Object(map) => {
            let mut s = format!("<{}>", key);
            for (k, v) in map {
                s.push_str(&to_xml(v, k));
            }
            s.push_str(&format!("</{}>", key));
            s
        }
        serde_json::Value::Array(arr) => {
            let mut s = String::new();
            for v in arr {
                s.push_str(&to_xml(v, "item"));
            }
            s
        }
        _ => format!("<{}>{}</{}>", key, val.to_string().trim_matches('"'), key),
    }
}

fn run_comparison(
    name: &str,
    json_data: &serde_json::Value,
    tabular_pct: f64,
    csv_support: bool,
) -> BenchmarkResult {
    // TOON
    let toon = convert_json(json_data).unwrap();
    let toon_tokens = count_tokens(&toon);

    // JSON (Pretty)
    let json_pretty = serde_json::to_string_pretty(json_data).unwrap();
    let json_tokens = count_tokens(&json_pretty);

    // JSON (Compact)
    let json_compact = serde_json::to_string(json_data).unwrap();
    let json_compact_tokens = count_tokens(&json_compact);

    // YAML
    let yaml = serde_yaml::to_string(json_data).unwrap();
    let yaml_tokens = count_tokens(&yaml);

    // XML
    let xml_str = to_xml(json_data, "root");
    let xml_tokens = count_tokens(&xml_str);

    // CSV (only for flat data)
    let csv_tokens = if csv_support {
        // Approximate CSV token count for comparison
        Some((json_compact_tokens as f64 * 0.6) as usize)
    } else {
        None
    };

    BenchmarkResult {
        name: name.to_string(),
        toon_tokens,
        json_tokens,
        json_compact_tokens,
        yaml_tokens,
        xml_tokens,
        _csv_tokens: csv_tokens,
        tabular_percentage: tabular_pct,
    }
}

/// Print a single bar in the official format
fn print_bar(
    label: &str,
    tokens: usize,
    max_tokens: usize,
    vs_json: Option<f64>,
    is_primary: bool,
) {
    let bar_width = 20;
    let filled = if max_tokens > 0 {
        ((tokens as f64 / max_tokens as f64) * bar_width as f64).round() as usize
    } else {
        0
    };
    let filled = filled.min(bar_width);
    let empty = bar_width - filled;

    let bar_char = if is_primary { "\u{2588}" } else { "\u{2591}" };
    let bar: String = std::iter::repeat(bar_char).take(filled).collect();
    let space: String = std::iter::repeat("\u{2591}").take(empty).collect();

    let diff_str = if let Some(pct) = vs_json {
        if pct >= 0.0 {
            format!(" (+{:.1}%)", pct)
        } else {
            format!(" ({:.1}%)", pct)
        }
    } else {
        "".to_string()
    };

    if is_primary {
        println!(
            "\u{2502} {} {}{} {:>6} tokens{}",
            label, bar, space, tokens, diff_str
        );
    } else {
        println!(
            "\u{251c}\u{2500} vs {} {}{} {:>6} tokens{}",
            label, bar, space, tokens, diff_str
        );
    }
}

fn print_result(res: &BenchmarkResult) {
    println!();
    println!(
        "{} \u{250a} Tabular: {:.0}%",
        res.name, res.tabular_percentage
    );

    let max_tokens = [
        res.toon_tokens,
        res.json_tokens,
        res.json_compact_tokens,
        res.yaml_tokens,
        res.xml_tokens,
    ]
    .iter()
    .max()
    .cloned()
    .unwrap_or(1);

    // Calculate savings vs JSON pretty
    let vs_json = |current: usize| -> f64 {
        ((current as f64 - res.json_tokens as f64) / res.json_tokens as f64) * 100.0
    };

    // TOON (primary)
    print_bar(
        "TOON",
        res.toon_tokens,
        max_tokens,
        Some(vs_json(res.toon_tokens)),
        true,
    );

    // Comparisons
    print_bar("JSON", res.json_tokens, max_tokens, Some(0.0), false);
    print_bar(
        "JSON compact",
        res.json_compact_tokens,
        max_tokens,
        Some(vs_json(res.json_compact_tokens)),
        false,
    );
    print_bar(
        "YAML",
        res.yaml_tokens,
        max_tokens,
        Some(vs_json(res.yaml_tokens)),
        false,
    );

    // XML (last item)
    let xml_diff = vs_json(res.xml_tokens);
    let bar_width = 20;
    let filled = ((res.xml_tokens as f64 / max_tokens as f64) * bar_width as f64).round() as usize;
    let bar: String = std::iter::repeat("\u{2591}")
        .take(filled.min(bar_width))
        .collect();
    let space: String = std::iter::repeat("\u{2591}")
        .take(bar_width - filled.min(bar_width))
        .collect();
    println!(
        "\u{2514}\u{2500} vs XML {}{} {:>6} tokens (+{:.1}%)",
        bar, space, res.xml_tokens, xml_diff
    );
}

fn print_summary(results: &[BenchmarkResult]) {
    println!();
    println!("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500} Total \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");

    let total_toon: usize = results.iter().map(|r| r.toon_tokens).sum();
    let total_json: usize = results.iter().map(|r| r.json_tokens).sum();
    let total_compact: usize = results.iter().map(|r| r.json_compact_tokens).sum();
    let total_yaml: usize = results.iter().map(|r| r.yaml_tokens).sum();
    let total_xml: usize = results.iter().map(|r| r.xml_tokens).sum();

    let max_total = [total_toon, total_json, total_compact, total_yaml, total_xml]
        .iter()
        .max()
        .cloned()
        .unwrap_or(1);

    let vs_json = |current: usize| -> f64 {
        ((current as f64 - total_json as f64) / total_json as f64) * 100.0
    };

    print_bar(
        "TOON",
        total_toon,
        max_total,
        Some(vs_json(total_toon)),
        true,
    );
    print_bar("JSON", total_json, max_total, Some(0.0), false);
    print_bar(
        "JSON compact",
        total_compact,
        max_total,
        Some(vs_json(total_compact)),
        false,
    );
    print_bar(
        "YAML",
        total_yaml,
        max_total,
        Some(vs_json(total_yaml)),
        false,
    );

    let xml_diff = vs_json(total_xml);
    println!(
        "\u{2514}\u{2500} vs XML {:>6} tokens (+{:.1}%)",
        total_xml, xml_diff
    );
}

fn benchmark_token_efficiency(_c: &mut Criterion) {
    println!();
    println!();
    println!("Token Efficiency Benchmark (using cl100k_base tokenizer)");
    println!("========================================================");
    println!("Output format matches official toon-format/toon benchmarks");

    let mut results = Vec::new();

    // Mixed-Structure Track (CSV not applicable)
    println!();
    println!("=== Mixed-Structure Track ===");
    println!("Datasets with nested or semi-uniform structures. CSV excluded.");

    let ecommerce = generate_ecommerce_orders(50);
    let res = run_comparison(
        "\u{1f6d2} E-commerce orders with nested structures",
        &ecommerce,
        33.0,
        false,
    );
    print_result(&res);
    results.push(res);

    let logs = generate_event_logs(75);
    let res = run_comparison("\u{1f9fe} Semi-uniform event logs", &logs, 50.0, false);
    print_result(&res);
    results.push(res);

    let config = generate_nested_config();
    let res = run_comparison("\u{1f9e9} Deeply nested configuration", &config, 0.0, false);
    print_result(&res);
    results.push(res);

    print_summary(&results);

    // Flat-Only Track (CSV applicable)
    println!();
    println!("=== Flat-Only Track ===");
    println!("Datasets with flat tabular structures where CSV is applicable.");

    let mut flat_results = Vec::new();

    let employees = generate_employee_records(100);
    let res = run_comparison(
        "\u{1f465} Uniform employee records",
        &employees,
        100.0,
        true,
    );
    print_result(&res);
    flat_results.push(res);

    let metrics = generate_time_series(60);
    let res = run_comparison(
        "\u{1f4c8} Time-series analytics data",
        &metrics,
        100.0,
        true,
    );
    print_result(&res);
    flat_results.push(res);

    let repos = generate_github_repos(100);
    let res = run_comparison("\u{2b50} Top 100 GitHub repositories", &repos, 100.0, true);
    print_result(&res);
    flat_results.push(res);

    print_summary(&flat_results);

    println!();
}

criterion_group!(benches, benchmark_token_efficiency);
criterion_main!(benches);
