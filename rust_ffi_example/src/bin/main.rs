use rust_ffi_example::{compress_rust_string, decompress_rust_data, encode_varint_rust, decode_varint_rust};
use std::env;
use std::fs;
use std::io::{self, Read};

fn print_usage(program_name: &str) {
    println!("Usage:");
    println!("  {} compress [text]              - Compress text (or from stdin)", program_name);
    println!("  {} decompress <file>            - Decompress binary file", program_name);
    println!("  {} encode-varint <number>         - Encode a u64 number into varint format (output as hex)", program_name);
    println!("  {} decode-varint <hex_bytes>      - Decode varint hex bytes into a u64 number", program_name);
    println!("  echo 'text' | {} compress       - Compress from stdin", program_name);
    println!("");
    println!("Examples:");
    println!("  {} compress \"Hello, world!\"", program_name);
    println!("  {} decompress compressed_output.bin", program_name);
    println!("  {} encode-varint 12345", program_name);
    println!("  {} decode-varint c96101", program_name);
    println!("  echo \"Hello from stdin\" | {} compress", program_name);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        return Ok(());
    }

    let operation = &args[1];

    match operation.as_str() {
        "compress" => {
            let input_data = if args.len() > 2 {
                // Use command line argument as input
                args[2].clone()
            } else {
                // Read from stdin
                println!("Reading from stdin... (press Ctrl+D when done)");
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            };

            if input_data.is_empty() {
                println!("No input data provided.");
                return Ok(());
            }

            println!("Original data length: {} bytes", input_data.len());

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
                    println!("To decompress: {} decompress {}", args[0], output_file);
                }
                Err(e) => {
                    eprintln!("Compression failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "decompress" => {
            if args.len() < 3 {
                eprintln!("Error: Decompress requires a file path.");
                print_usage(&args[0]);
                std::process::exit(1);
            }

            let file_path = &args[2];

            // Read compressed data from file
            let compressed_data = match fs::read(file_path) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", file_path, e);
                    std::process::exit(1);
                }
            };

            println!("Compressed data length: {} bytes", compressed_data.len());

            // Decompress the data (original size is read automatically from header)
            match decompress_rust_data(&compressed_data) {
                Ok(decompressed_string) => {
                    println!("Decompressed data length: {} bytes", decompressed_string.len());
                    println!("Decompressed data: \"{}\"", decompressed_string);
                    
                    // Write decompressed data to file
                    let output_file = "decompressed_output.txt";
                    fs::write(output_file, &decompressed_string)?;
                    println!("Decompressed data written to: {}", output_file);
                }
                Err(e) => {
                    eprintln!("Decompression failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "encode-varint" => {
            if args.len() < 3 {
                eprintln!("Error: encode-varint requires a number.");
                print_usage(&args[0]);
                std::process::exit(1);
            }
            let number_str = &args[2];
            match number_str.parse::<u64>() {
                Ok(number) => {
                    match encode_varint_rust(number) {
                        Ok(encoded_bytes) => {
                            let hex_string: String = encoded_bytes
                                .iter()
                                .map(|&b| format!("{:02x}", b))
                                .collect();
                            println!("{}", hex_string);
                        }
                        Err(e) => {
                            eprintln!("Error encoding varint: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Error: Invalid number format '{}'. Please provide a valid u64 number.", number_str);
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
        }
        "decode-varint" => {
            if args.len() < 3 {
                eprintln!("Error: decode-varint requires hex bytes.");
                print_usage(&args[0]);
                std::process::exit(1);
            }
            let hex_str = &args[2];
            match hex::decode(hex_str) {
                Ok(bytes) => {
                    match decode_varint_rust(&bytes) {
                        Ok((decoded_number, bytes_read)) => {
                            println!("Decoded number: {}", decoded_number);
                            println!("Bytes read: {}", bytes_read);
                        }
                        Err(e) => {
                            eprintln!("Error decoding varint: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Error: Invalid hex string '{}'.", hex_str);
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
            }
        }
        // This is the new position for the default arm
        _ => {
            eprintln!("Error: Unknown operation '{}'. Use 'compress', 'decompress', 'encode-varint', or 'decode-varint'.", operation);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
} 