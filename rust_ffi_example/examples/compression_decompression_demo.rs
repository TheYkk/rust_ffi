use std::env;
use std::fs;

use rust_ffi_example::{
    compress_rust_string, decompress_rust_data,
    compress_rust_string_lz4, decompress_rust_data_lz4
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
    match compress_rust_string(&file_contents) {
        Ok(compressed) => {
            println!("Compressed length: {} bytes", compressed.len());
            
            if file_contents.len() > 0 {
                let ratio = (compressed.len() as f64 / file_contents.len() as f64) * 100.0;
                println!("Compression ratio: {:.2}%", ratio);
                
                if ratio < 100.0 {
                    println!("✅ Compression effective");
                } else {
                    println!("⚠️  Compression ineffective");
                }
            }
            
            let hex_preview: String = compressed.iter().take(20).map(|&b| format!("{:02x}", b)).collect::<Vec<String>>().join(" ");
            println!("Compressed hex (first 20 bytes): {}", hex_preview);
            
            // Decompress back to verify
            match decompress_rust_data(&compressed) {
                Ok(decompressed) => {
                    println!("Decompressed length: {} bytes", decompressed.len());
                    
                    if file_contents == decompressed {
                        println!("✅ ZLIB Round-trip successful!\n");
                    } else {
                        println!("❌ ZLIB Round-trip failed!\n");
                    }
                }
                Err(e) => {
                    println!("❌ ZLIB Decompression failed: {}\n", e);
                }
            }
        }
        Err(e) => {
            println!("❌ ZLIB Compression failed: {}\n", e);
        }
    }

    // Test LZ4 compression
    println!("--- LZ4 Compression/Decompression ---");
    match compress_rust_string_lz4(&file_contents) {
        Ok(compressed) => {
            println!("Compressed length: {} bytes", compressed.len());

            if file_contents.len() > 0 {
                let ratio = (compressed.len() as f64 / file_contents.len() as f64) * 100.0;
                println!("Compression ratio: {:.2}%", ratio);

                if ratio < 100.0 {
                    println!("✅ Compression effective");
                } else {
                    println!("⚠️  Compression ineffective");
                }
            }

            let hex_preview: String = compressed.iter().take(20).map(|&b| format!("{:02x}", b)).collect::<Vec<String>>().join(" ");
            println!("Compressed hex (first 20 bytes): {}", hex_preview);

            // Decompress back to verify
            match decompress_rust_data_lz4(&compressed) {
                Ok(decompressed) => {
                    println!("Decompressed length: {} bytes", decompressed.len());

                    if file_contents == decompressed {
                        println!("✅ LZ4 Round-trip successful!");
                    } else {
                        println!("❌ LZ4 Round-trip failed!");
                    }
                }
                Err(e) => {
                    println!("❌ LZ4 Decompression failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ LZ4 Compression failed: {}", e);
        }
    }
    
    println!("\n=== Demo Complete ===");
    Ok(())
}