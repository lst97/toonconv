use criterion::{Criterion, criterion_group, criterion_main, BenchmarkId};
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;
use toonconv::conversion::ConversionConfig;
use std::path::Path;

fn benchmark_directory_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_processing");

    let sizes = vec![10, 100, 500];

    for size in sizes {
        let tmp = tempdir().unwrap();
        for i in 0..size {
            let fpath = tmp.path().join(format!("file{}.json", i));
            let mut f = File::create(fpath).unwrap();
            write!(f, "{{\"id\": {}, \"name\": \"user{}\"}}\n", i, i).unwrap();
        }

        group.bench_with_input(BenchmarkId::new("dir_size", size), &tmp.path(), |b, path| {
            let output_dir = tempdir().unwrap();
            b.iter(|| {
                let args = [path.to_str().unwrap(), "--output", output_dir.path().to_str().unwrap()];
                let _ = std::process::Command::new("cargo").args(&["run", "--bin", "toonconv", "--"]).args(&args).output().unwrap();
            })
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_directory_processing);
criterion_main!(benches);
