use rust_ffi_example::compress_rust_string;
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Simple usage: program [text_to_compress] or program < input_file
    let input_data = if args.len() > 1 {
        // Use command line argument as input
        args[1].clone()
    } else {
        // Read from stdin
        println!("Reading from stdin... (press Ctrl+D when done)");
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    if input_data.is_empty() {
        println!("Usage: {} [text_to_compress]", args[0]);
        println!("   or: echo 'text' | {}", args[0]);
        return Ok(());
    }

    println!("Original data length: {} bytes", input_data.len());
    println!("Original data: \"{}\"", input_data.trim());

    // Compress the data
    match compress_rust_string(&input_data) {
        Ok(compressed_data) => {
            println!("Compressed data length: {} bytes", compressed_data.len());
            println!(
                "Compression ratio: {:.2}%",
                (compressed_data.len() as f64 / input_data.len() as f64) * 100.0
            );
            
            // Show first few bytes of compressed data as hex
            let hex_preview: String = compressed_data
                .iter()
                .take(16)
                .map(|&b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ");
            println!("Compressed data (first 16 bytes as hex): {}", hex_preview);
            
            // Write compressed data to file
            let output_file = "compressed_output.bin";
            fs::write(output_file, &compressed_data)?;
            println!("Compressed data written to: {}", output_file);
        }
        Err(e) => {
            eprintln!("Compression failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
} 