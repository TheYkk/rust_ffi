# Rust FFI Compression Library

A Rust library that provides string compression and decompression functionality through a C FFI interface using zlib. This project demonstrates how to safely wrap C libraries in Rust while providing comprehensive testing, benchmarking, and fuzzing capabilities.

## Features

- **Safe FFI Wrapper**: Rust wrapper around a C compression/decompression library using zlib
- **Auto-sizing Decompression**: Compressed data includes original size header - no need to specify length manually
- **CLI Binary**: Command-line tool for compressing and decompressing text/files
- **Comprehensive Benchmarks**: Performance testing with various input sizes and patterns
- **Fuzzing Support**: Property-based and fuzz testing for robustness
- **Property-based Testing**: Structured testing with the `arbitrary` crate

## Compressed Data Format

The library uses an enhanced compression format with space-efficient Variable-Byte Encoding:

```
[varint original length header][zlib compressed data]
```

**Variable-Byte Encoding (Varint) Details:**
- **1 byte** for lengths 0-127 (most common case)
- **2 bytes** for lengths 128-16,383
- **3 bytes** for lengths 16,384-2,097,151  
- **4 bytes** for lengths 2,097,152-268,435,455
- **5 bytes** for lengths 268,435,456+

**Benefits:**
- **Space-efficient**: Uses only 1-2 bytes for typical string lengths instead of fixed 8 bytes
- **User-friendly**: No need to manually specify original length during decompression
- **Self-contained**: Compressed files contain all necessary information
- **Error prevention**: Eliminates mistakes from incorrect length parameters
- **Industry standard**: Varint encoding used in Protocol Buffers, SQLite, and other formats

**Note**: This format is not backward compatible with data compressed using earlier versions.

## Dependencies

This project requires zlib to be installed on your system:

### macOS
```bash
brew install zlib
```

### Ubuntu/Debian
```bash
sudo apt-get install zlib1g-dev
```

### CentOS/RHEL
```bash
sudo yum install zlib-devel
```

## Building

```bash
cargo build
```

For release build:
```bash
cargo build --release
```

## Usage

### Library

```rust
use rust_ffi_example::{compress_rust_string, decompress_rust_data};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = "Hello, world! This string will be compressed and decompressed.";
    
    // Compress the string
    match compress_rust_string(original) {
        Ok(compressed) => {
            println!("Original size: {} bytes", original.len());
            println!("Compressed size: {} bytes", compressed.len());
            println!("Compression ratio: {:.2}%", 
                (compressed.len() as f64 / original.len() as f64) * 100.0);
            
            // Decompress the data back to string
            match decompress_rust_data(&compressed) {
                Ok(decompressed) => {
                    println!("Decompressed: '{}'", decompressed);
                    assert_eq!(original, decompressed, "Round trip should preserve data");
                }
                Err(e) => eprintln!("Decompression failed: {}", e),
            }
        }
        Err(e) => eprintln!("Compression failed: {}", e),
    }
    
    Ok(())
}
```

### CLI Binary

Build and run the compression CLI:

```bash
cargo build --release
```

**Compress a string directly:**
```bash
./target/release/compression_cli compress "Hello, world! This is a test string."
```

**Compress from stdin:**
```bash
echo "Hello from stdin" | ./target/release/compression_cli compress
```

**Compress a file:**
```bash
cat some_file.txt | ./target/release/compression_cli compress
```

**Decompress a compressed file:**
```bash
./target/release/compression_cli decompress compressed_output.bin
```

The CLI will output:
- For compression: Original data and length, compressed data length, compression ratio, hex preview of compressed data (showing varint header), and save compressed data to `compressed_output.bin`
- For decompression: Compressed data length, decompressed data, and save decompressed data to `decompressed_output.txt` (original size is automatically detected from varint header)

## Testing

### Basic Tests
```bash
cargo test
```

### Property-based Tests
The library includes property-based tests that verify:
- Compression determinism
- Error handling for invalid inputs
- Unicode string support
- Special character handling

Run all tests including property tests:
```bash
cargo test property_tests
```

## Benchmarking

Run performance benchmarks:
```bash
cargo bench
```

This will run comprehensive benchmarks testing:

### Benchmark Categories

1. **Size-based benchmarks**: Testing compression performance with different input sizes (100B to 100KB)
2. **Pattern-based benchmarks**: Testing different types of content:
   - Highly repetitive data
   - Moderately repetitive data  
   - Random text
   - Mixed content
3. **Small string benchmarks**: Edge cases with empty and small strings
4. **Edge case benchmarks**: Special patterns like all zeros, alternating characters
5. **Real-world data benchmarks**: JSON, log files, and code-like content

### Benchmark Results

Results are saved to `target/criterion/` with HTML reports available at:
```
target/criterion/report/index.html
```

## Fuzzing

This project includes comprehensive fuzzing support using `cargo-fuzz`.

### Setup Fuzzing

Fuzzing requires the nightly Rust toolchain. First, install it:
```bash
rustup install nightly
```

Then install cargo-fuzz (if not already installed):
```bash
cargo install cargo-fuzz
```

### Run Fuzzing

**Basic fuzzing (runs indefinitely until stopped with Ctrl+C):**
```bash
cargo +nightly fuzz run fuzz_compression
```

**Fuzzing with timeout:**
```bash
cargo +nightly fuzz run fuzz_compression -- -max_total_time=60
```

**Fuzzing with specific options:**
```bash
cargo +nightly fuzz run fuzz_compression -- -max_len=10000 -jobs=4
```

### Fuzzing Features

The fuzzer tests:
- Valid and invalid UTF-8 sequences
- Strings with embedded null bytes (should fail gracefully)
- Various string patterns and lengths
- Memory safety and error handling
- Edge cases and boundary conditions

**Quiet Fuzzing Mode**: By default, the fuzzer runs quietly without error message noise from expected failure cases. This allows the fuzzer to efficiently test millions of inputs without cluttering the output.

**Verbose Error Messages**: For debugging purposes, you can enable detailed error messages by building with the `verbose-errors` feature:
```bash
cargo build --features verbose-errors
cargo +nightly fuzz run fuzz_decompress --features verbose-errors
```

This will show detailed error messages for:
- Invalid compressed data (too small, corrupted headers)
- Varint decoding failures
- Zlib decompression errors
- Memory allocation failures

### Fuzzing Results

- **Corpus**: Input samples that triggered new code paths are saved to `fuzz/corpus/fuzz_compression/`
- **Crashes**: Any crashes or panics are saved to `fuzz/artifacts/fuzz_compression/`
- **Coverage**: Track code coverage to ensure thorough testing

**Performance**: The fuzzer can process hundreds of thousands to millions of test cases per second, efficiently finding edge cases and potential vulnerabilities.

## Error Handling

The library handles several error conditions gracefully:

- **Null bytes in input**: Returns `Err("Failed to create CString, input might contain null bytes")`
- **Compression failure**: Returns `Err("Compression failed in C library (null buffer returned)")`
- **Memory allocation failure**: Handled by the C library

## Memory Safety

This library ensures memory safety through:
- Proper CString conversion and null byte checking
- Automatic cleanup of C-allocated memory using `free_compressed_data`
- Safe pointer handling with null checks
- Bounds checking on all buffer operations

## Performance Characteristics

Based on benchmark results:
- **Highly repetitive data**: Excellent compression ratios (often 90%+ reduction)
- **Random data**: Lower compression ratios but still effective
- **Small strings**: Fixed overhead from zlib headers (may increase size)
- **Large data**: Better compression ratios due to amortized header costs

## Contributing

1. Ensure all tests pass: `cargo test`
2. Run benchmarks to check performance: `cargo bench`
3. Run fuzzing for a reasonable time: `cargo fuzz run fuzz_compression`
4. Update documentation as needed

## Security

This library:
- ✅ Handles null bytes safely
- ✅ Prevents buffer overflows
- ✅ Manages C memory properly
- ✅ Validates all inputs
- ✅ Uses fuzzing for security testing

## License

This project is licensed under the MIT License - see the LICENSE file for details. 