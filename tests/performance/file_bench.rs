use criterion::{Criterion, criterion_group, criterion_main, BenchmarkId};
use tempfile::NamedTempFile;
use std::io::Write;
use toonconv::conversion::{convert_json_string, convert_stream_to_toon, ConversionConfig};

fn generate_large_file_content(size: usize) -> String {
    let mut s = String::new();
    s.push('[');
    let mut i = 0;
    while s.len() < size {
        if i > 0 { s.push(','); }
        s.push_str(&format!(r#"{{"id": {}, "name": "user{}"}}"#, i, i));
        i += 1;
    }
    s.push(']');
    s
}

fn file_processing_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_processing");

    let sizes = vec![50_000, 100_000, 500_000];
    for size in sizes {
        let content = generate_large_file_content(size);
        group.bench_with_input(BenchmarkId::new("file_size", size), &content, |b, content| {
            b.iter(|| {
                let mut tmp = NamedTempFile::new().unwrap();
                write!(tmp, "{}", content).unwrap();
                tmp.flush().unwrap();
                // Use streaming conversion
                let config = ConversionConfig::large_files();
                let _ = convert_stream_to_toon(std::fs::File::open(tmp.path()).unwrap(), &config).unwrap();
            })
        });
    }

    group.finish();
}

criterion_group!(benches, file_processing_bench);
criterion_main!(benches);
