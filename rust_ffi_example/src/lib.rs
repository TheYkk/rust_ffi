use std::ffi::CString;
use std::os::raw::{c_char, c_ulong};
use std::slice;

// Define the Rust equivalent of the C struct CompressedData
#[repr(C)]
pub struct CompressedData {
    pub buffer: *mut c_char,
    pub length: c_ulong,
}

// Declare the C functions that will be called from Rust
extern "C" {
    pub fn compress_string(input: *const c_char, input_len: c_ulong) -> CompressedData;
    pub fn free_compressed_data(data: CompressedData);
}

/// Compresses a string using the C library's `compress_string` function.
///
/// # Arguments
/// * `s`: The string slice to compress.
///
/// # Returns
/// * `Ok(Vec<u8>)` containing the compressed data if successful.
/// * `Err(&str)` with an error message if compression fails or input is invalid.
///
/// # Safety
/// This function wraps unsafe FFI calls. It handles C string conversion
/// and memory management for the data returned by the C function.
pub fn compress_rust_string(s: &str) -> Result<Vec<u8>, &'static str> {
    // Convert the Rust string to a C-compatible string (null-terminated)
    let c_input_string = match CString::new(s) {
        Ok(cs) => cs,
        Err(_) => return Err("Failed to create CString, input might contain null bytes"),
    };

    // Get a pointer to the C string's raw data
    let input_ptr = c_input_string.as_ptr();
    // Length of the string (excluding the null terminator for compress_string)
    let input_len = s.len() as c_ulong;

    // Call the C function
    // This is an unsafe block because we are calling C code and dealing with raw pointers.
    let compressed_c_data = unsafe { compress_string(input_ptr, input_len) };

    // Check if the C function returned a valid buffer
    if compressed_c_data.buffer.is_null() {
        // The C function should have printed an error, but we also return an error here.
        // Note: No need to call free_compressed_data if buffer is null.
        return Err("Compression failed in C library (null buffer returned)");
    }

    // Convert the C data (raw pointer and length) to a Rust Vec<u8>
    // This is also unsafe because we are dereferencing a raw pointer from C.
    let rust_vec: Vec<u8> = unsafe {
        // Create a slice from the raw parts
        let slice = slice::from_raw_parts(compressed_c_data.buffer as *const u8, compressed_c_data.length as usize);
        // Clone the data into a new Vec<u8>
        slice.to_vec()
    };

    // Free the memory allocated by the C function
    // This is crucial to prevent memory leaks.
    unsafe {
        free_compressed_data(compressed_c_data);
    }

    Ok(rust_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_basic() {
        let original_data = "This is a test string for zlib compression, hopefully it gets smaller.";
        println!("Original data: '{}'", original_data);
        println!("Original length: {}", original_data.len());

        match compress_rust_string(original_data) {
            Ok(compressed_data) => {
                println!("Compressed length: {}", compressed_data.len());
                // For very short strings, compression might not reduce size.
                // For a reasonably long string, it should.
                assert!(compressed_data.len() > 0, "Compressed data should not be empty.");
                if original_data.len() > 20 { // Only assert smaller if original is somewhat long
                    assert!(compressed_data.len() < original_data.len(), "Compressed data should be smaller than original for this input.");
                }

                // To actually verify, we would need a decompress function.
                // For now, we're just checking that it ran and changed the data size.
                // Example: print first few bytes of compressed data
                // println!("Compressed data (first 10 bytes as hex): {:?}", &compressed_data.iter().take(10).map(|&b| format!("{:02x}", b)).collect::<Vec<String>>());
            }
            Err(e) => {
                panic!("test_compression_basic failed: {}", e);
            }
        }
    }

    #[test]
    fn test_compression_empty_string() {
        let original_data = "";
        println!("Original data: '{}'", original_data);
        println!("Original length: {}", original_data.len());

        match compress_rust_string(original_data) {
            Ok(compressed_data) => {
                println!("Compressed length for empty string: {}", compressed_data.len());
                // zlib compressing an empty string results in a small, fixed-size output
                assert!(compressed_data.len() > 0, "Compressed empty string should not be empty.");
            }
            Err(e) => {
                panic!("test_compression_empty_string failed: {}", e);
            }
        }
    }

    #[test]
    fn test_string_with_null_byte_internal() {
        // CString::new will fail for strings with interior null bytes.
        let original_data = "hello\0world";
        // We expect compress_rust_string to return an Err here.
        assert!(compress_rust_string(original_data).is_err(), "Should fail for string with internal null byte due to CString conversion.");
    }
}
