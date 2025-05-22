use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use rust_ffi_example::{encode_varint_rust, decode_varint_rust};

fn bench_encode_varint_by_value_size(c: &mut Criterion) {
    let test_values = vec![
        ("1_byte", vec![0, 1, 50, 127]),
        ("2_byte", vec![128, 255, 1000, 16383]),
        ("3_byte", vec![16384, 32000, 100000, 2097151]),
        ("4_byte", vec![2097152, 10000000, 100000000, 268435455]),
        ("5_byte", vec![268435456, 1000000000, 10000000000u64, 34359738367]),
        ("8_byte", vec![1u64 << 40, 1u64 << 50, 1u64 << 60, u64::MAX]),
    ];
    
    let mut group = c.benchmark_group("encode_varint_by_size");
    
    for (size_name, values) in test_values {
        for &value in &values {
            group.bench_with_input(
                BenchmarkId::new(size_name, value),
                &value,
                |b, &value| {
                    b.iter(|| {
                        encode_varint_rust(black_box(value)).unwrap()
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_decode_varint_by_length(c: &mut Criterion) {
    // Pre-encode test values to benchmark decoding
    let test_cases = vec![
        ("1_byte", vec![0, 1, 50, 127]),
        ("2_byte", vec![128, 255, 1000, 16383]),
        ("3_byte", vec![16384, 32000, 100000, 2097151]),
        ("4_byte", vec![2097152, 10000000, 100000000, 268435455]),
        ("5_byte", vec![268435456, 1000000000, 10000000000u64, 34359738367]),
        ("8_byte", vec![1u64 << 40, 1u64 << 50, 1u64 << 60, u64::MAX]),
    ];
    
    let mut group = c.benchmark_group("decode_varint_by_length");
    
    for (size_name, values) in test_cases {
        for &value in &values {
            let encoded = encode_varint_rust(value).unwrap();
            group.bench_with_input(
                BenchmarkId::new(size_name, value),
                &encoded,
                |b, encoded| {
                    b.iter(|| {
                        decode_varint_rust(black_box(encoded)).unwrap()
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_varint_roundtrip(c: &mut Criterion) {
    let test_values = vec![
        0, 1, 127, 128, 255, 256, 16383, 16384, 65535, 65536,
        1u64 << 20, 1u64 << 30, 1u64 << 40, 1u64 << 50, 1u64 << 60, u64::MAX,
    ];
    
    let mut group = c.benchmark_group("varint_roundtrip");
    
    for &value in &test_values {
        group.bench_with_input(
            BenchmarkId::new("roundtrip", value),
            &value,
            |b, &value| {
                b.iter(|| {
                    let encoded = encode_varint_rust(black_box(value)).unwrap();
                    let (decoded, _) = decode_varint_rust(black_box(&encoded)).unwrap();
                    assert_eq!(value, decoded);
                    decoded
                });
            },
        );
    }
    group.finish();
}

fn bench_varint_throughput(c: &mut Criterion) {
    // Test throughput with different data patterns
    let patterns = vec![
        ("sequential_small", (0..1000).collect::<Vec<u64>>()),
        ("sequential_medium", (0..1000).map(|i| i * 1000).collect::<Vec<u64>>()),
        ("sequential_large", (0..1000).map(|i| (i as u64) << 20).collect::<Vec<u64>>()),
        ("powers_of_2", (0..64).map(|i| 1u64 << i).cycle().take(1000).collect::<Vec<u64>>()),
        ("random_pattern", {
            let mut vals = Vec::new();
            for i in 0..1000 {
                vals.push((i * 31 + 17) % (1u64 << 32)); // Pseudo-random pattern
            }
            vals
        }),
    ];
    
    let mut group = c.benchmark_group("varint_throughput");
    
    for (pattern_name, values) in patterns {
        // Benchmark encoding throughput
        group.throughput(Throughput::Elements(values.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("encode", pattern_name),
            &values,
            |b, values| {
                b.iter(|| {
                    for &value in values {
                        let _ = encode_varint_rust(black_box(value)).unwrap();
                    }
                });
            },
        );
        
        // Pre-encode for decoding benchmark
        let encoded_values: Vec<Vec<u8>> = values.iter()
            .map(|&v| encode_varint_rust(v).unwrap())
            .collect();
        
        // Benchmark decoding throughput
        group.bench_with_input(
            BenchmarkId::new("decode", pattern_name),
            &encoded_values,
            |b, encoded_values| {
                b.iter(|| {
                    for encoded in encoded_values {
                        let _ = decode_varint_rust(black_box(encoded)).unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_varint_edge_cases(c: &mut Criterion) {
    let edge_cases = vec![
        ("min_value", 0),
        ("max_1_byte", 127),
        ("min_2_byte", 128),
        ("max_2_byte", 16383),
        ("min_3_byte", 16384),
        ("min_4_byte", 2097152),
        ("min_5_byte", 268435456),
        ("large_value", 1u64 << 50),
        ("max_value", u64::MAX),
    ];
    
    let mut group = c.benchmark_group("varint_edge_cases");
    
    for (case_name, value) in edge_cases {
        // Encode benchmark
        group.bench_function(&format!("encode_{}", case_name), |b| {
            b.iter(|| {
                encode_varint_rust(black_box(value)).unwrap()
            });
        });
        
        // Decode benchmark
        let encoded = encode_varint_rust(value).unwrap();
        group.bench_function(&format!("decode_{}", case_name), |b| {
            b.iter(|| {
                decode_varint_rust(black_box(&encoded)).unwrap()
            });
        });
        
        // Round-trip benchmark
        group.bench_function(&format!("roundtrip_{}", case_name), |b| {
            b.iter(|| {
                let encoded = encode_varint_rust(black_box(value)).unwrap();
                let (decoded, _) = decode_varint_rust(black_box(&encoded)).unwrap();
                assert_eq!(value, decoded);
                decoded
            });
        });
    }
    group.finish();
}

fn bench_varint_decode_with_extra_data(c: &mut Criterion) {
    // Test decoding performance when there's extra data after the varint
    let base_values = vec![0, 127, 128, 16383, 16384, u64::MAX];
    let extra_data_sizes = vec![0, 1, 10, 100, 1000];
    
    let mut group = c.benchmark_group("varint_decode_extra_data");
    
    for &value in &base_values {
        let encoded = encode_varint_rust(value).unwrap();
        
        for &extra_size in &extra_data_sizes {
            let mut data_with_extra = encoded.clone();
            data_with_extra.extend(vec![0x42u8; extra_size]); // Add extra bytes
            
            group.bench_with_input(
                BenchmarkId::new(format!("value_{}_extra_{}", value, extra_size), extra_size),
                &data_with_extra,
                |b, data| {
                    b.iter(|| {
                        let (decoded, bytes_read) = decode_varint_rust(black_box(data)).unwrap();
                        assert_eq!(value, decoded);
                        assert_eq!(bytes_read, encoded.len());
                        decoded
                    });
                },
            );
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_encode_varint_by_value_size,
    bench_decode_varint_by_length,
    bench_varint_roundtrip,
    bench_varint_throughput,
    bench_varint_edge_cases,
    bench_varint_decode_with_extra_data
);
criterion_main!(benches); 