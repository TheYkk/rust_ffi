use rust_ffi_example::{compress_rust_string, decompress_rust_data};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust FFI Compression/Decompression Demo ===\n");
    
    let repetitive_string = "a".repeat(100);
    let test_cases = vec![
        "Hello, world!",
        "This is a longer string that should compress better due to its length and repetitive patterns.",
        "🦀 Rust FFI with Unicode! 🌟 Testing émojis and spëcial characters: café, naïve, résumé 🎉",
        &repetitive_string, // Highly repetitive string
        "", // Empty string edge case
    ];
    
    for (i, original) in test_cases.iter().enumerate() {
        println!("--- Test Case {} ---", i + 1);
        println!("Original: \"{}\"", original);
        println!("Original length: {} bytes", original.len());
        
        // Compress the string
        match compress_rust_string(original) {
            Ok(compressed) => {
                println!("Compressed length: {} bytes", compressed.len());
                
                if original.len() > 0 {
                    let ratio = (compressed.len() as f64 / original.len() as f64) * 100.0;
                    println!("Compression ratio: {:.2}%", ratio);
                    
                    if ratio < 100.0 {
                        println!("✅ Compression effective (smaller than original)");
                    } else {
                        println!("⚠️  Compression ineffective (larger than original)");
                    }
                } else {
                    println!("📦 Empty string compressed to zlib header");
                }
                
                // Show hex representation of compressed data (first 20 bytes)
                let hex_preview: String = compressed
                    .iter()
                    .take(20)
                    .map(|&b| format!("{:02x}", b))
                    .collect::<Vec<String>>()
                    .join(" ");
                println!("Compressed hex (first 20 bytes): {}", hex_preview);
                
                // Decompress back to verify
                match decompress_rust_data(&compressed) {
                    Ok(decompressed) => {
                        println!("Decompressed: \"{}\"", decompressed);
                        println!("Decompressed length: {} bytes", decompressed.len());
                        
                        if original == &decompressed {
                            println!("✅ Round-trip successful! Data preserved.");
                        } else {
                            println!("❌ Round-trip failed! Data corrupted.");
                            println!("Expected: \"{}\"", original);
                            println!("Got:      \"{}\"", decompressed);
                        }
                    }
                    Err(e) => {
                        println!("❌ Decompression failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Compression failed: {}", e);
            }
        }
        
        println!(); // Empty line for readability
    }
    
    println!("=== Demo Complete ===");
    Ok(())
} 