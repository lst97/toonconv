use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::json;
use toonconv::convert_json;

fn estimate_tokens(s: &str) -> usize {
    // Very naive tokenizer for token-count benchmark: split on whitespace & punctuation
    s.split(|c: char| c.is_whitespace() || ",:.{}[]()".contains(c))
        .filter(|t| !t.is_empty())
        .count()
}

fn token_count_bench(c: &mut Criterion) {
    // Create a sample JSON object with a large array of uniform objects
    let mut users = Vec::new();
    for i in 0..1000 {
        users.push(json!({
            "id": i,
            "name": format!("User{}", i),
            "active": i % 2 == 0
        }));
    }

    let json = json!({"users": users});
    let json_str = serde_json::to_string(&json).unwrap();

    c.bench_function("token_count/json_to_toon", |b| {
        b.iter(|| {
            let toon = convert_json(&json).unwrap();
            let json_tokens = estimate_tokens(&json_str);
            let toon_tokens = estimate_tokens(&toon);
            // Sanity-check: token reduction should be positive
            assert!(toon_tokens <= json_tokens);
        })
    });
}

criterion_group!(benches, token_count_bench);
criterion_main!(benches);
