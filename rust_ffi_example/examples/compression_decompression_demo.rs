use std::env;
use std::fs;
use std::time::Instant;

use rust_ffi_example::{
    compress_rust_string, decompress_rust_data,
    compress_rust_string_lz4, decompress_rust_data_lz4,
    compress_rust_string_zstd, decompress_rust_data_zstd
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get filename from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    println!("=== Rust FFI Compression Demo for: {} ===\n", file_path);
    
    let file_contents = fs::read_to_string(file_path)?;
    // println!("Original content: \"{}\"", file_contents);
    println!("Original length: {} bytes\n", file_contents.len());

    // Test ZLIB compression
    println!("--- ZLIB Compression/Decompression ---");
    let start_compress_zlib = Instant::now();
    match compress_rust_string(&file_contents) {
        Ok(compressed_zlib) => {
            let compress_duration_zlib = start_compress_zlib.elapsed();
            let mut zlib_compression_ratio = 0.0;
            if file_contents.len() > 0 {
                zlib_compression_ratio = (compressed_zlib.len() as f64 / file_contents.len() as f64) * 100.0;
            }
            print!("ZLIB: Original size: {}, Compressed size: {}, Compression ratio: {:.2}%, Time to compress: {:.2?}", 
                   file_contents.len(), compressed_zlib.len(), zlib_compression_ratio, compress_duration_zlib);

            let start_decompress_zlib = Instant::now();
            match decompress_rust_data(&compressed_zlib) {
                Ok(decompressed_zlib) => {
                    let decompress_duration_zlib = start_decompress_zlib.elapsed();
                    println!(", Time to decompress: {:.2?}", decompress_duration_zlib);
                    if file_contents == decompressed_zlib {
                        println!("✅ ZLIB Round-trip successful!");
                    } else {
                        println!("❌ ZLIB Round-trip failed!");
                    }
                }
                Err(e) => {
                    println!("\n❌ ZLIB Decompression failed: {}", e);
                }
            }
        }
        Err(e) => {
            let compress_duration_zlib = start_compress_zlib.elapsed();
            println!("ZLIB: Time to compress (failed): {:.2?}", compress_duration_zlib);
            println!("❌ ZLIB Compression failed: {}", e);
        }
    }
    println!(); // Add a newline for better separation

    // Test LZ4 compression
    println!("--- LZ4 Compression/Decompression ---");
    let start_compress_lz4 = Instant::now();
    match compress_rust_string_lz4(&file_contents) {
        Ok(compressed_lz4) => {
            let compress_duration_lz4 = start_compress_lz4.elapsed();
            let mut lz4_compression_ratio = 0.0;
            if file_contents.len() > 0 {
                lz4_compression_ratio = (compressed_lz4.len() as f64 / file_contents.len() as f64) * 100.0;
            }
            print!("LZ4: Original size: {}, Compressed size: {}, Compression ratio: {:.2}%, Time to compress: {:.2?}", 
                   file_contents.len(), compressed_lz4.len(), lz4_compression_ratio, compress_duration_lz4);

            let start_decompress_lz4 = Instant::now();
            match decompress_rust_data_lz4(&compressed_lz4) {
                Ok(decompressed_lz4) => {
                    let decompress_duration_lz4 = start_decompress_lz4.elapsed();
                    println!(", Time to decompress: {:.2?}", decompress_duration_lz4);
                    if file_contents == decompressed_lz4 {
                        println!("✅ LZ4 Round-trip successful!");
                    } else {
                        println!("❌ LZ4 Round-trip failed!");
                    }
                }
                Err(e) => {
                    println!("\n❌ LZ4 Decompression failed: {}", e);
                }
            }
        }
        Err(e) => {
            let compress_duration_lz4 = start_compress_lz4.elapsed();
            println!("LZ4: Time to compress (failed): {:.2?}", compress_duration_lz4);
            println!("❌ LZ4 Compression failed: {}", e);
        }
    }
    println!(); // Add a newline for better separation

    // Test ZSTD compression
    println!("--- Zstandard (zstd) Compression/Decompression ---");
    let start_compress_zstd = Instant::now();
    match compress_rust_string_zstd(&file_contents) {
        Ok(compressed_zstd) => {
            let compress_duration_zstd = start_compress_zstd.elapsed();
            let mut zstd_compression_ratio = 0.0;
            if file_contents.len() > 0 {
                zstd_compression_ratio = (compressed_zstd.len() as f64 / file_contents.len() as f64) * 100.0;
            }
            print!("ZSTD: Original size: {}, Compressed size: {}, Compression ratio: {:.2}%, Time to compress: {:.2?}", 
                   file_contents.len(), compressed_zstd.len(), zstd_compression_ratio, compress_duration_zstd);
            
            let start_decompress_zstd = Instant::now();
            match decompress_rust_data_zstd(&compressed_zstd) {
                Ok(decompressed_zstd) => {
                    let decompress_duration_zstd = start_decompress_zstd.elapsed();
                    println!(", Time to decompress: {:.2?}", decompress_duration_zstd);
                    if file_contents == decompressed_zstd {
                        println!("✅ ZSTD Round-trip successful!");
                    } else {
                        println!("❌ ZSTD Round-trip failed!");
                    }
                }
                Err(e) => {
                    println!("\n❌ ZSTD Decompression failed: {}", e);
                }
            }
        }
        Err(e) => {
            let compress_duration_zstd = start_compress_zstd.elapsed();
            println!("ZSTD: Time to compress (failed): {:.2?}", compress_duration_zstd);
            println!("❌ ZSTD Compression failed: {}", e);
        }
    }
    
    println!("\n=== Demo Complete ===");
    Ok(())
}