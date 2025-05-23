use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use rust_ffi_example::{compress_rust_string, decompress_rust_data};

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
        group.bench_function(name, |b| { // Changed to bench_function for consistency if not using BenchmarkId with value
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
    
    for (name, data_str) in test_cases { // Renamed data to data_str to avoid conflict if we take its length for throughput
        group.throughput(Throughput::Bytes(data_str.len() as u64));
        group.bench_function(name, |b| { // Changed to bench_function
            b.iter(|| {
                compress_rust_string(black_box(&data_str)).unwrap()
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
    
    for (name, data_str) in test_cases { // Renamed data to data_str
        group.throughput(Throughput::Bytes(data_str.len() as u64));
        group.bench_function(name, |b| { // Changed to bench_function
            b.iter(|| {
                compress_rust_string(black_box(&data_str)).unwrap()
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
    bench_real_world_data,
    // Decompression benchmarks
    bench_decompression_by_size,
    bench_decompression_by_pattern,
    bench_decompression_small_strings,
    bench_decompression_edge_cases,
    bench_decompression_real_world_data
);
criterion_main!(benches); 


// --- Decompression Benchmarks ---

fn bench_decompression_by_size(c: &mut Criterion) {
    let sizes = vec![100, 1000, 10000, 100000];
    let test_pattern = "This is a test string that should compress well with zlib. ";
    
    let mut group = c.benchmark_group("decompression_by_size");
    
    for size in sizes {
        let original_data = generate_test_data(size, test_pattern);
        let compressed_data = compress_rust_string(&original_data).expect("Compression failed during benchmark setup");
        
        group.throughput(Throughput::Bytes(original_data.len() as u64)); // Throughput is original size
        group.bench_with_input(
            BenchmarkId::new("decompress", size),
            &compressed_data,
            |b, data| {
                b.iter(|| {
                    decompress_rust_data(black_box(data)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_decompression_by_pattern(c: &mut Criterion) {
    let size = 10000; // Original data size
    let patterns = vec![
        ("highly_repetitive", "AAAAAAAAAA"),
        ("moderately_repetitive", "Hello world! "),
        ("random_text", "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6"),
        ("mixed_content", "The quick brown fox jumps over the lazy dog. 1234567890!@#$%^&*()"),
    ];
    
    let mut group = c.benchmark_group("decompression_by_pattern");
    
    for (name, pattern) in patterns {
        let original_data = generate_test_data(size, pattern);
        let compressed_data = compress_rust_string(&original_data).expect("Compression failed during benchmark setup");

        group.throughput(Throughput::Bytes(original_data.len() as u64)); // Throughput is original size
        group.bench_with_input(
            BenchmarkId::new("decompress", name),
            &compressed_data,
            |b, data| {
                b.iter(|| {
                    decompress_rust_data(black_box(data)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_decompression_small_strings(c: &mut Criterion) {
    let test_cases_original: Vec<(&str, String)> = vec![
        ("empty", "".to_string()),
        ("single_char", "A".to_string()),
        ("small_string", "Hello".to_string()),
        ("medium_string", "The quick brown fox jumps over the lazy dog".to_string()),
    ];
    
    let mut group = c.benchmark_group("decompression_small_strings");
    
    for (name, original_data_str) in test_cases_original {
        let compressed_data = compress_rust_string(&original_data_str).expect("Compression failed during benchmark setup");
        
        // Using BenchmarkId here for consistency with other decompression benches if passing compressed_data
        group.bench_with_input(BenchmarkId::new("decompress", name), &compressed_data, |b, data| {
             b.iter(|| {
                decompress_rust_data(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_decompression_edge_cases(c: &mut Criterion) {
    let test_cases_original: Vec<(&str, String)> = vec![
        ("all_ones", "1".repeat(1000)),
        ("alternating", "01".repeat(500)),
        ("increasing", (0..1000).map(|i| ((i % 26) as u8 + b'a') as char).collect::<String>()),
        ("all_spaces", " ".repeat(1000)),
    ];
    
    let mut group = c.benchmark_group("decompression_edge_cases");
    
    for (name, original_data) in test_cases_original {
        let compressed_data = compress_rust_string(&original_data).expect("Compression failed during benchmark setup");
        group.throughput(Throughput::Bytes(original_data.len() as u64)); 
        
        group.bench_with_input(BenchmarkId::new("decompress", name), &compressed_data, |b, data| {
            b.iter(|| {
                decompress_rust_data(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_decompression_real_world_data(c: &mut Criterion) {
    let json_like_original = r#"{"name":"John","age":30,"city":"New York","hobbies":["reading","swimming","coding"],"address":{"street":"123 Main St","zip":"10001"}}"#.repeat(100);
    let log_like_original = "[2023-01-01 12:00:00] INFO: Application started successfully\n[2023-01-01 12:00:01] DEBUG: Loading configuration file\n[2023-01-01 12:00:02] WARN: Configuration file not found, using defaults\n".repeat(50);
    let code_like_original = "fn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n    let y = x * 2;\n    println!(\"Result: {}\", y);\n}\n".repeat(100);
    
    let test_cases_original = vec![
        ("json_data", json_like_original),
        ("log_data", log_like_original),
        ("code_data", code_like_original),
    ];
    
    let mut group = c.benchmark_group("decompression_real_world_data");
    
    for (name, original_data) in test_cases_original {
        let compressed_data = compress_rust_string(&original_data).expect("Compression failed during benchmark setup");
        group.throughput(Throughput::Bytes(original_data.len() as u64));
        
        group.bench_with_input(BenchmarkId::new("decompress", name), &compressed_data, |b, data| {
            b.iter(|| {
                decompress_rust_data(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}