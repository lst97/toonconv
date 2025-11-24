use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::json;

fn bench_serde(c: &mut Criterion) {
    let mut users = Vec::new();
    for i in 0..10_000 {
        users.push(json!({
            "id": i,
            "name": format!("User{}", i),
            "email": format!("user{}@example.com", i),
            "active": i % 2 == 0
        }));
    }

    let json = json!({"users": users});
    let json_str = serde_json::to_string(&json).unwrap();

    c.bench_function("simd_eval/serde_from_str", |b| {
        b.iter(|| {
            let _ = serde_json::from_str::<serde_json::Value>(&json_str).unwrap();
        })
    });
}

#[cfg(feature = "simd")]
fn bench_simd(c: &mut Criterion) {
    use simd_json::to_borrowed_value;

    let mut users = Vec::new();
    for i in 0..10_000 {
        users.push(json!({
            "id": i,
            "name": format!("User{}", i),
            "email": format!("user{}@example.com", i),
            "active": i % 2 == 0
        }));
    }

    let json = json!({"users": users});
    let mut json_bytes = serde_json::to_vec(&json).unwrap();

    c.bench_function("simd_eval/simd_from_slice", |b| {
        b.iter(|| {
            // simd-json requires mutable byte slice
            let mut clone = json_bytes.clone();
            let _ = to_borrowed_value(&mut clone).unwrap();
        })
    });
}

#[cfg(not(feature = "simd"))]
fn bench_simd(_c: &mut Criterion) {
    // Simd not compiled; skip
}

fn simd_group(c: &mut Criterion) {
    bench_serde(c);
    bench_simd(c);
}

criterion_group!(benches, simd_group);
criterion_main!(benches);
