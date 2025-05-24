use rust_ffi_example::{
    compress_rust_string, decompress_rust_data,
    compress_rust_string_lz4, decompress_rust_data_lz4
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust FFI Compression/Decompression Demo ===\n");
    
    let repetitive_string_zlib = "a".repeat(100);
    let repetitive_string_lz4 = "b".repeat(120); // Use a slightly different one for LZ4 for clarity
    let zlib_test_cases = vec![
        "Hello, Zlib World!",
        "This is a longer zlib string that should compress better due to its length and repetitive patterns.",
        "🦀 Zlib: Rust FFI with Unicode! 🌟 Testing émojis and spëcial characters: café, naïve, résumé 🎉",
        &repetitive_string_zlib, // Highly repetitive string
        "", // Empty string edge case for Zlib
    ];

    println!("--- ZLIB Compression/Decompression ---");
    for (i, original) in zlib_test_cases.iter().enumerate() {
        println!("\n--- Zlib Test Case {} ---", i + 1);
        println!("Zlib Original: \"{}\"", original);
        println!("Zlib Original length: {} bytes", original.len());
        
        // Compress the string using Zlib
        match compress_rust_string(original) {
            Ok(compressed) => {
                println!("Zlib Compressed length: {} bytes", compressed.len());
                
                if original.len() > 0 {
                    let ratio = (compressed.len() as f64 / original.len() as f64) * 100.0;
                    println!("Zlib Compression ratio: {:.2}%", ratio);
                    
                    if ratio < 100.0 {
                        println!("✅ Zlib Compression effective (smaller than original)");
                    } else {
                        println!("⚠️  Zlib Compression ineffective (larger than original or same size)");
                    }
                } else {
                    println!("📦 Zlib: Empty string compressed");
                }
                
                let hex_preview: String = compressed.iter().take(20).map(|&b| format!("{:02x}", b)).collect::<Vec<String>>().join(" ");
                println!("Zlib Compressed hex (first 20 bytes): {}", hex_preview);
                
                // Decompress back to verify
                match decompress_rust_data(&compressed) {
                    Ok(decompressed) => {
                        println!("Zlib Decompressed: \"{}\"", decompressed);
                        println!("Zlib Decompressed length: {} bytes", decompressed.len());
                        
                        if original == &decompressed {
                            println!("✅ Zlib Round-trip successful! Data preserved.");
                        } else {
                            println!("❌ Zlib Round-trip failed! Data corrupted.");
                            println!("Expected: \"{}\"", original);
                            println!("Got:      \"{}\"", decompressed);
                        }
                    }
                    Err(e) => {
                        println!("❌ Zlib Decompression failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Zlib Compression failed: {}", e);
            }
        }
    }

    println!("\n\n--- LZ4 Compression/Decompression ---");
    let lz4_test_cases = vec![
        "Hello, LZ4 World!",
        "This is a longer LZ4 string that should compress very well with LZ4 due to its speedy algorithm.",
        "🦀 LZ4: Rust FFI with Unicode! 🌟 Testing émojis and spëcial characters: café, naïve, résumé 🎉 (LZ4 version)",
        &repetitive_string_lz4, // Highly repetitive string for LZ4
        "", // Empty string edge case for LZ4
    ];

    for (i, original) in lz4_test_cases.iter().enumerate() {
        println!("\n--- LZ4 Test Case {} ---", i + 1);
        println!("LZ4 Original: \"{}\"", original);
        println!("LZ4 Original length: {} bytes", original.len());

        // Compress the string using LZ4
        match compress_rust_string_lz4(original) {
            Ok(compressed) => {
                println!("LZ4 Compressed length: {} bytes", compressed.len());

                if original.len() > 0 {
                    let ratio = (compressed.len() as f64 / original.len() as f64) * 100.0;
                    println!("LZ4 Compression ratio: {:.2}%", ratio);

                    if ratio < 100.0 {
                        println!("✅ LZ4 Compression effective (smaller than original)");
                    } else {
                        println!("⚠️  LZ4 Compression ineffective (larger than original or same size)");
                    }
                } else {
                    println!("📦 LZ4: Empty string compressed");
                }

                let hex_preview: String = compressed.iter().take(20).map(|&b| format!("{:02x}", b)).collect::<Vec<String>>().join(" ");
                println!("LZ4 Compressed hex (first 20 bytes): {}", hex_preview);

                // Decompress back to verify
                match decompress_rust_data_lz4(&compressed) {
                    Ok(decompressed) => {
                        println!("LZ4 Decompressed: \"{}\"", decompressed);
                        println!("LZ4 Decompressed length: {} bytes", decompressed.len());

                        if original == &decompressed {
                            println!("✅ LZ4 Round-trip successful! Data preserved.");
                        } else {
                            println!("❌ LZ4 Round-trip failed! Data corrupted.");
                            println!("Expected: \"{}\"", original);
                            println!("Got:      \"{}\"", decompressed);
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
    }
    
    println!("\n=== Demo Complete ===");
    Ok(())
}