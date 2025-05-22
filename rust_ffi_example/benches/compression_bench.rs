use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use rust_ffi_example::compress_rust_string;

fn generate_test_data(size: usize, pattern: &str) -> String {
    pattern.repeat(size / pattern.len() + 1)[..size].to_string()
}

fn bench_compression_by_size(c: &mut Criterion) {
    let sizes = vec![100, 1000, 10000, 100000];
    let test_pattern = "This is a test string that should compress well with zlib. ";
    
    let mut group = c.benchmark_group("compression_by_size");
    
    for size in sizes {
        let data = generate_test_data(size, test_pattern);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("compress", size),
            &data,
            |b, data| {
                b.iter(|| {
                    compress_rust_string(black_box(data)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_compression_by_pattern(c: &mut Criterion) {
    let size = 10000;
    let patterns = vec![
        ("highly_repetitive", "AAAAAAAAAA"),
        ("moderately_repetitive", "Hello world! "),
        ("random_text", "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6"),
        ("mixed_content", "The quick brown fox jumps over the lazy dog. 1234567890!@#$%^&*()"),
    ];
    
    let mut group = c.benchmark_group("compression_by_pattern");
    group.throughput(Throughput::Bytes(size as u64));
    
    for (name, pattern) in patterns {
        let data = generate_test_data(size, pattern);
        group.bench_with_input(
            BenchmarkId::new("compress", name),
            &data,
            |b, data| {
                b.iter(|| {
                    compress_rust_string(black_box(data)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_empty_and_small_strings(c: &mut Criterion) {
    let test_cases = vec![
        ("empty", ""),
        ("single_char", "A"),
        ("small_string", "Hello"),
        ("medium_string", "The quick brown fox jumps over the lazy dog"),
    ];
    
    let mut group = c.benchmark_group("small_strings");
    
    for (name, data) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                compress_rust_string(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_compression_edge_cases(c: &mut Criterion) {
    let test_cases = vec![
        ("all_ones", "1".repeat(1000)),
        ("alternating", "01".repeat(500)),
        ("increasing", (0..1000).map(|i| ((i % 26) as u8 + b'a') as char).collect::<String>()),
        ("all_spaces", " ".repeat(1000)),
    ];
    
    let mut group = c.benchmark_group("edge_cases");
    
    for (name, data) in test_cases {
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                compress_rust_string(black_box(&data)).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_real_world_data(c: &mut Criterion) {
    // Simulate different types of real-world data
    let json_like = r#"{"name":"John","age":30,"city":"New York","hobbies":["reading","swimming","coding"],"address":{"street":"123 Main St","zip":"10001"}}"#.repeat(100);
    let log_like = "[2023-01-01 12:00:00] INFO: Application started successfully\n[2023-01-01 12:00:01] DEBUG: Loading configuration file\n[2023-01-01 12:00:02] WARN: Configuration file not found, using defaults\n".repeat(50);
    let code_like = "fn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n    let y = x * 2;\n    println!(\"Result: {}\", y);\n}\n".repeat(100);
    
    let test_cases = vec![
        ("json_data", json_like),
        ("log_data", log_like),
        ("code_data", code_like),
    ];
    
    let mut group = c.benchmark_group("real_world_data");
    
    for (name, data) in test_cases {
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                compress_rust_string(black_box(&data)).unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_compression_by_size,
    bench_compression_by_pattern,
    bench_empty_and_small_strings,
    bench_compression_edge_cases,
    bench_real_world_data
);
criterion_main!(benches); 