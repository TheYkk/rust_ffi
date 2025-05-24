use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use rust_ffi_example::{
    compress_rust_string, decompress_rust_data,
    compress_rust_string_lz4, decompress_rust_data_lz4
};

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
    bench_decompression_real_world_data,
    // LZ4 Benchmarks
    bench_lz4_compression_by_size,
    bench_lz4_compression_by_pattern,
    bench_lz4_empty_and_small_strings,
    bench_lz4_compression_edge_cases,
    bench_lz4_real_world_data,
    bench_lz4_decompression_by_size,
    bench_lz4_decompression_by_pattern,
    bench_lz4_decompression_small_strings,
    bench_lz4_decompression_edge_cases,
    bench_lz4_decompression_real_world_data
);
criterion_main!(benches);


// --- Zlib Decompression Benchmarks ---

fn bench_decompression_by_size(c: &mut Criterion) {
    let sizes = vec![100, 1000, 10000, 100000];
    let test_pattern = "This is a test string that should compress well with zlib. ";
    
    let mut group = c.benchmark_group("zlib_decompression_by_size"); // Renamed group
    
    for size in sizes {
        let original_data = generate_test_data(size, test_pattern);
        let compressed_data = compress_rust_string(&original_data).expect("Zlib compression failed during benchmark setup");
        
        group.throughput(Throughput::Bytes(original_data.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("zlib_decompress", size), // Renamed BenchmarkId
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
    
    let mut group = c.benchmark_group("zlib_decompression_by_pattern"); // Renamed group
    
    for (name, pattern) in patterns {
        let original_data = generate_test_data(size, pattern);
        let compressed_data = compress_rust_string(&original_data).expect("Zlib compression failed during benchmark setup");

        group.throughput(Throughput::Bytes(original_data.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("zlib_decompress", name), // Renamed BenchmarkId
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
    
    let mut group = c.benchmark_group("zlib_decompression_small_strings"); // Renamed group
    
    for (name, original_data_str) in test_cases_original {
        let compressed_data = compress_rust_string(&original_data_str).expect("Zlib compression failed during benchmark setup");
        
        group.bench_with_input(BenchmarkId::new("zlib_decompress", name), &compressed_data, |b, data| { // Renamed BenchmarkId
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
    
    let mut group = c.benchmark_group("zlib_decompression_edge_cases"); // Renamed group
    
    for (name, original_data) in test_cases_original {
        let compressed_data = compress_rust_string(&original_data).expect("Zlib compression failed during benchmark setup");
        group.throughput(Throughput::Bytes(original_data.len() as u64)); 
        
        group.bench_with_input(BenchmarkId::new("zlib_decompress", name), &compressed_data, |b, data| { // Renamed BenchmarkId
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
    
    let mut group = c.benchmark_group("zlib_decompression_real_world_data"); // Renamed group
    
    for (name, original_data) in test_cases_original {
        let compressed_data = compress_rust_string(&original_data).expect("Zlib compression failed during benchmark setup");
        group.throughput(Throughput::Bytes(original_data.len() as u64));
        
        group.bench_with_input(BenchmarkId::new("zlib_decompress", name), &compressed_data, |b, data| { // Renamed BenchmarkId
            b.iter(|| {
                decompress_rust_data(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}


// --- LZ4 Compression Benchmarks ---

fn bench_lz4_compression_by_size(c: &mut Criterion) {
    let sizes = vec![100, 1000, 10000, 100000];
    let test_pattern = "This is a test string that should compress well with LZ4. "; // Slightly different for clarity
    
    let mut group = c.benchmark_group("lz4_compression_by_size");
    
    for size in sizes {
        let data = generate_test_data(size, test_pattern);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("lz4_compress", size),
            &data,
            |b, data| {
                b.iter(|| {
                    compress_rust_string_lz4(black_box(data)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_lz4_compression_by_pattern(c: &mut Criterion) {
    let size = 10000;
    let patterns = vec![
        ("highly_repetitive_lz4", "BBBBBBBBBB"), // Different pattern for LZ4
        ("moderately_repetitive_lz4", "Goodbye moon! "), // Different pattern
        ("random_text_lz4", "z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g0f9e8d7c6b5a4"),
        ("mixed_content_lz4", "A sleepy brown cat yawns quietly near the warm fireplace. 0987654321)(*&^%$#@!"),
    ];
    
    let mut group = c.benchmark_group("lz4_compression_by_pattern");
    group.throughput(Throughput::Bytes(size as u64));
    
    for (name, pattern) in patterns {
        let data = generate_test_data(size, pattern);
        group.bench_with_input(
            BenchmarkId::new("lz4_compress", name),
            &data,
            |b, data| {
                b.iter(|| {
                    compress_rust_string_lz4(black_box(data)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_lz4_empty_and_small_strings(c: &mut Criterion) {
    let test_cases = vec![
        ("lz4_empty", ""),
        ("lz4_single_char", "B"),
        ("lz4_small_string", "Hiya"),
        ("lz4_medium_string", "The lazy cat sleeps on the warm mat by the window"),
    ];
    
    let mut group = c.benchmark_group("lz4_small_strings");
    
    for (name, data) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                compress_rust_string_lz4(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_lz4_compression_edge_cases(c: &mut Criterion) {
    let test_cases = vec![
        ("lz4_all_twos", "2".repeat(1000)),
        ("lz4_alternating_ab", "ab".repeat(500)),
        ("lz4_decreasing", (0..1000).map(|i| (((999 - i) % 26) as u8 + b'a') as char).collect::<String>()),
        ("lz4_all_newlines", "\n".repeat(1000)),
    ];
    
    let mut group = c.benchmark_group("lz4_edge_cases");
    
    for (name, data_str) in test_cases {
        group.throughput(Throughput::Bytes(data_str.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                compress_rust_string_lz4(black_box(&data_str)).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_lz4_real_world_data(c: &mut Criterion) {
    let json_like_lz4 = r#"{"id":101,"item":"LZ4 Widget","status":"active","details":{"color":"blue","weight_kg":0.75},"inventory":{"stock":1500,"warehouse":"B"}}"#.repeat(90);
    let log_like_lz4 = "[TIME_LZ4] EVENT: User logged_in, id=user_xyz\n[TIME_LZ4] ACTION: File_download, name=document.lz4\n[TIME_LZ4] WARN: Low_disk_space, path=/mnt/data\n".repeat(55);
    let code_like_lz4 = "struct Lz4Processor<T> {\n    data: Vec<T>,\n    capacity: usize,\n}\nimpl<T> Lz4Processor<T> {\n    fn new(capacity: usize) -> Self {\n        Lz4Processor { data: Vec::with_capacity(capacity), capacity }\n    }\n}\n".repeat(95);
    
    let test_cases = vec![
        ("lz4_json_data", json_like_lz4),
        ("lz4_log_data", log_like_lz4),
        ("lz4_code_data", code_like_lz4),
    ];
    
    let mut group = c.benchmark_group("lz4_real_world_data");
    
    for (name, data_str) in test_cases {
        group.throughput(Throughput::Bytes(data_str.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                compress_rust_string_lz4(black_box(&data_str)).unwrap()
            });
        });
    }
    group.finish();
}

// --- LZ4 Decompression Benchmarks ---

fn bench_lz4_decompression_by_size(c: &mut Criterion) {
    let sizes = vec![100, 1000, 10000, 100000];
    let test_pattern = "This is an LZ4 test string. LZ4 is fast. "; // LZ4 specific pattern
    
    let mut group = c.benchmark_group("lz4_decompression_by_size");
    
    for size in sizes {
        let original_data = generate_test_data(size, test_pattern);
        let compressed_data = compress_rust_string_lz4(&original_data).expect("LZ4 compression failed during benchmark setup");
        
        group.throughput(Throughput::Bytes(original_data.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("lz4_decompress", size),
            &compressed_data,
            |b, data| {
                b.iter(|| {
                    decompress_rust_data_lz4(black_box(data)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_lz4_decompression_by_pattern(c: &mut Criterion) {
    let size = 10000; // Original data size
    let patterns = vec![
        ("highly_repetitive_lz4", "BBBBBBBBBB"),
        ("moderately_repetitive_lz4", "Goodbye moon! "),
        ("random_text_lz4", "z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g0f9e8d7c6b5a4"),
        ("mixed_content_lz4", "A sleepy brown cat yawns quietly near the warm fireplace. 0987654321)(*&^%$#@!"),
    ];
    
    let mut group = c.benchmark_group("lz4_decompression_by_pattern");
    
    for (name, pattern) in patterns {
        let original_data = generate_test_data(size, pattern);
        let compressed_data = compress_rust_string_lz4(&original_data).expect("LZ4 compression failed during benchmark setup");

        group.throughput(Throughput::Bytes(original_data.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("lz4_decompress", name),
            &compressed_data,
            |b, data| {
                b.iter(|| {
                    decompress_rust_data_lz4(black_box(data)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_lz4_decompression_small_strings(c: &mut Criterion) {
    let test_cases_original: Vec<(&str, String)> = vec![
        ("lz4_empty", "".to_string()),
        ("lz4_single_char", "B".to_string()),
        ("lz4_small_string", "Hiya".to_string()),
        ("lz4_medium_string", "The lazy cat sleeps on the warm mat by the window".to_string()),
    ];
    
    let mut group = c.benchmark_group("lz4_decompression_small_strings");
    
    for (name, original_data_str) in test_cases_original {
        let compressed_data = compress_rust_string_lz4(&original_data_str).expect("LZ4 compression failed during benchmark setup");
        
        group.bench_with_input(BenchmarkId::new("lz4_decompress", name), &compressed_data, |b, data| {
             b.iter(|| {
                decompress_rust_data_lz4(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_lz4_decompression_edge_cases(c: &mut Criterion) {
    let test_cases_original: Vec<(&str, String)> = vec![
        ("lz4_all_twos", "2".repeat(1000)),
        ("lz4_alternating_ab", "ab".repeat(500)),
        ("lz4_decreasing", (0..1000).map(|i| (((999 - i) % 26) as u8 + b'a') as char).collect::<String>()),
        ("lz4_all_newlines", "\n".repeat(1000)),
    ];
    
    let mut group = c.benchmark_group("lz4_decompression_edge_cases");
    
    for (name, original_data) in test_cases_original {
        let compressed_data = compress_rust_string_lz4(&original_data).expect("LZ4 compression failed during benchmark setup");
        group.throughput(Throughput::Bytes(original_data.len() as u64)); 
        
        group.bench_with_input(BenchmarkId::new("lz4_decompress", name), &compressed_data, |b, data| {
            b.iter(|| {
                decompress_rust_data_lz4(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_lz4_decompression_real_world_data(c: &mut Criterion) {
    let json_like_original_lz4 = r#"{"id":101,"item":"LZ4 Widget","status":"active","details":{"color":"blue","weight_kg":0.75},"inventory":{"stock":1500,"warehouse":"B"}}"#.repeat(90);
    let log_like_original_lz4 = "[TIME_LZ4] EVENT: User logged_in, id=user_xyz\n[TIME_LZ4] ACTION: File_download, name=document.lz4\n[TIME_LZ4] WARN: Low_disk_space, path=/mnt/data\n".repeat(55);
    let code_like_original_lz4 = "struct Lz4Processor<T> {\n    data: Vec<T>,\n    capacity: usize,\n}\nimpl<T> Lz4Processor<T> {\n    fn new(capacity: usize) -> Self {\n        Lz4Processor { data: Vec::with_capacity(capacity), capacity }\n    }\n}\n".repeat(95);
    
    let test_cases_original = vec![
        ("lz4_json_data", json_like_original_lz4),
        ("lz4_log_data", log_like_original_lz4),
        ("lz4_code_data", code_like_original_lz4),
    ];
    
    let mut group = c.benchmark_group("lz4_decompression_real_world_data");
    
    for (name, original_data) in test_cases_original {
        let compressed_data = compress_rust_string_lz4(&original_data).expect("LZ4 compression failed during benchmark setup");
        group.throughput(Throughput::Bytes(original_data.len() as u64));
        
        group.bench_with_input(BenchmarkId::new("lz4_decompress", name), &compressed_data, |b, data| {
            b.iter(|| {
                decompress_rust_data_lz4(black_box(data)).unwrap()
            });
        });
    }
    group.finish();
}