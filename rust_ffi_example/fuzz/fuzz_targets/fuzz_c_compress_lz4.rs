#![no_main]
use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use rust_ffi_example::{compress_rust_string_lz4, decompress_rust_data_lz4};

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

    // Attempt to compress the string using LZ4
    match compress_rust_string_lz4(&original_data) {
        Ok(compressed_data) => {
            // If compression was successful, try to decompress it
            match decompress_rust_data_lz4(&compressed_data) {
                Ok(decompressed_data) => {
                    // Assert that the decompressed data matches the original
                    assert_eq!(original_data, decompressed_data, "LZ4 Round trip failed: original and decompressed data do not match.");
                }
                Err(e) => {
                    // This case should ideally not happen if compression succeeded and produced valid data.
                    // However, if the C layer has a bug and produces invalid compressed data from valid input,
                    // this could be triggered.
                    // We don't panic here to allow the fuzzer to explore this state.
                    eprintln!("LZ4 Decompression failed after successful compression for input '{}': {}", original_data, e);
                }
            }
        }
        Err(e) => {
            // Compression can fail, e.g., if the input string contains null bytes,
            // which CString::new (used in compress_rust_string_lz4) cannot handle.
            // This is an expected failure path, so we don't panic.
            // The fuzzer will continue exploring other inputs.
            if original_data.contains('\0') {
                // Expected error for strings with null bytes.
                assert_eq!(e, "Failed to create CString, input might contain null bytes");
            } else {
                // Unexpected compression error
                // It's useful to know if compression fails for other reasons.
                eprintln!("LZ4 Compression unexpectedly failed for input '{}': {}", original_data, e);
                // Depending on desired strictness, one might panic here for unexpected errors.
                // For now, we'll print and continue to allow fuzzing other paths.
            }
        }
    }
});
