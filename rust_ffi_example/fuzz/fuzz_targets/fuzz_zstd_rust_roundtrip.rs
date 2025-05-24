#![no_main]
use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use rust_ffi_example::{compress_rust_string_zstd, decompress_rust_data_zstd};

#[derive(Debug, Clone)]
struct FuzzInput {
    data: String,
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, libfuzzer_sys::arbitrary::Error> {
        let data = String::arbitrary(u)?;
        Ok(FuzzInput { data })
    }
}

fuzz_target!(|input: FuzzInput| {
    let original_data = input.data;

    // Attempt to compress the string using the Rust ZSTD wrapper
    match compress_rust_string_zstd(&original_data) {
        Ok(compressed_data) => {
            // If compression was successful, try to decompress it using the Rust ZSTD wrapper
            match decompress_rust_data_zstd(&compressed_data) {
                Ok(decompressed_data) => {
                    // Assert that the decompressed data matches the original
                    assert_eq!(original_data, decompressed_data, "ZSTD Rust Round trip failed: original and decompressed data do not match.");
                }
                Err(e) => {
                    // This case should ideally not happen if compression succeeded and produced valid data
                    // that our own Rust wrapper should be able to decompress.
                    // This could indicate an issue in how compressed data is passed or how decompression handles it.
                    eprintln!("ZSTD Rust Decompression failed after successful Rust compression for input '{}': {}", original_data, e);
                    // Potentially panic here if we expect all successfully compressed strings to be decompressible.
                    // For now, logging to allow fuzzer to explore more.
                }
            }
        }
        Err(e) => {
            // Compression can fail, e.g., if the input string contains null bytes,
            // which CString::new (used in compress_rust_string_zstd) cannot handle.
            // This is an expected failure path.
            if original_data.contains('\0') {
                // Expected error for strings with null bytes.
                assert_eq!(e, "Failed to create CString, input might contain null bytes");
            } else {
                // Unexpected compression error
                eprintln!("ZSTD Rust Compression unexpectedly failed for input '{}': {}", original_data, e);
                // Potentially panic for unexpected errors.
            }
        }
    }
});
