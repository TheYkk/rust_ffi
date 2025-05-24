# Example Rust FFI with C Library

This project demonstrates how to call C code from Rust using Foreign Function Interface (FFI). It includes examples of:
- Compressing and decompressing data using C libraries (Zlib, LZ4, and Zstandard).
- Variable-byte integer encoding and decoding.
- Building C code using a `build.rs` script.
- Exposing C functions to Rust and Rust functions to C (though the latter is not extensively used in this example).
- Unit testing and benchmarking for both Rust and C components.
- Fuzz testing for the FFI boundary.

## Compression Libraries

This crate provides FFI bindings to C libraries for data compression and decompression. Zlib, LZ4, and Zstandard (zstd) are supported. The compressed data format includes a custom header: `[varint encoded original length][actual compressed data]`.

### Zlib

- **Description**: Uses the Zlib library for DEFLATE-based compression and decompression.
- **Rust Wrappers**:
    - `compress_rust_string(input: &str) -> Result<Vec<u8>, &str>`
    - `decompress_rust_data(input: &[u8]) -> Result<String, &str>`
- **Underlying C Functions**:
    - `CompressedData compress_string(const char *input, unsigned long input_len)`
    - `DecompressedData decompress_data(const char *input, unsigned long input_len)`

### LZ4

- **Description**: Uses the LZ4 library for fast compression and decompression. This was recently added as an alternative to Zlib.
- **Rust Wrappers**:
    - `compress_rust_string_lz4(input: &str) -> Result<Vec<u8>, &str>`
    - `decompress_rust_data_lz4(input: &[u8]) -> Result<String, &str>`
- **Underlying C Functions**:
    - `CompressedData compress_string_lz4(const char *input, unsigned long input_len)`
    - `DecompressedData decompress_data_lz4(const char *input, unsigned long input_len)`

### Zstandard (zstd)

- **Description**: Uses the Zstandard library for high-performance compression and decompression.
- **Rust Wrappers**:
    - `compress_rust_string_zstd(input: &str) -> Result<Vec<u8>, &str>`
    - `decompress_rust_data_zstd(input: &[u8]) -> Result<String, &str>`
- **Underlying C Functions**:
    - `CompressedData compress_string_zstd(const char *input, unsigned long input_len)`
    - `DecompressedData decompress_data_zstd(const char *input, unsigned long input_len)`

## Variable-Byte Encoding

The project also includes C functions for variable-byte encoding (`encode_varint`) and decoding (`decode_varint`) of unsigned long integers. These are used internally by the compression functions to prefix the compressed data with the original data's length.
- **Rust Wrappers**:
    - `encode_varint_rust(value: u64) -> Result<Vec<u8>, &str>`
    - `decode_varint_rust(data: &[u8]) -> Result<(u64, usize), &str>`

## Building and Dependencies

The C code (`src/clib.c`) is compiled and linked by the `build.rs` script.
- **`pkg-config`**: The build script uses `pkg-config` to locate `zlib`, `liblz4` (the LZ4 library), and `libzstd` (the Zstandard library) on the system. This is the preferred method for finding the necessary compilation and linking flags.
- **Fallback**: If `pkg-config` fails to find any library (e.g., `pkg-config` is not installed, or the `.pc` files for the libraries are not in `pkg-config`'s search path), the build script will attempt to link them directly (e.g., using `-lz` for zlib, `-llz4` for LZ4, and `-lzstd` for Zstandard).
- **Requirements**: For a successful build and for all features (including compression, decompression, tests, benchmarks, and fuzzing) to work correctly, you should have the development libraries for Zlib, LZ4, and Zstandard installed. These packages provide the necessary header files (like `zlib.h`, `lz4.h`, and `zstd.h`) and shared library objects.
    - On Debian/Ubuntu: `sudo apt-get install zlib1g-dev liblz4-dev libzstd-dev`
    - On Fedora: `sudo dnf install zlib-devel lz4-devel zstd-devel`
    - On macOS (using Homebrew): `brew install lz4 zstd` (zlib is often pre-installed or available through Xcode Command Line Tools; if not, `brew install zlib` might be needed).

## Testing

- **Unit Tests**: Run with `cargo test`. This includes tests for Rust functions which in turn call the C FFI functions for Zlib, LZ4, and Zstandard.
- **Benchmarks**: Run with `cargo bench`. Benchmarks for Zlib, LZ4, and Zstandard compression/decompression are available.
- **Fuzzing**: Fuzz targets are defined in the `fuzz/` directory. See the `rust_ffi_example/fuzz/README.md` (if available) or `cargo-fuzz` documentation for instructions on how to run them. New fuzz targets for LZ4 (`fuzz_c_compress_lz4`, `fuzz_c_decompress_lz4`) and Zstandard (`fuzz_c_compress_zstd`, `fuzz_c_decompress_zstd`, `fuzz_zstd_rust_roundtrip`) have been added.

## Examples

Check the `examples/` directory. To run the demonstration:
`cargo run --example compression_decompression_demo`
This demo now includes Zlib, LZ4, and Zstandard operations.
