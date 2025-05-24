use std::ffi::CString;
use std::os::raw::{c_char, c_ulong};
use std::slice;

// Define the Rust equivalent of the C struct CompressedData
#[repr(C)]
pub struct CompressedData {
    pub buffer: *mut c_char,
    pub length: c_ulong,
}

// Define the Rust equivalent of the C struct DecompressedData
#[repr(C)]
pub struct DecompressedData {
    pub buffer: *mut c_char,
    pub length: c_ulong,
}

// Declare the C functions that will be called from Rust
extern "C" {
    pub fn compress_string(input: *const c_char, input_len: c_ulong) -> CompressedData;
    pub fn free_compressed_data(data: CompressedData);
    pub fn decompress_data(input: *const c_char, input_len: c_ulong) -> DecompressedData;
    pub fn free_decompressed_data(data: DecompressedData);

    // LZ4 functions
    pub fn compress_string_lz4(input: *const c_char, input_len: c_ulong) -> CompressedData;
    pub fn decompress_data_lz4(input: *const c_char, input_len: c_ulong) -> DecompressedData;
    
    // Variable-byte encoding functions
    pub fn encode_varint(value: c_ulong, buffer: *mut c_char) -> i32;
    pub fn decode_varint(buffer: *const c_char, max_bytes: i32, value: *mut c_ulong) -> i32;
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

/// Decompresses data using the C library's `decompress_data` function.
/// The original size is automatically read from the compressed data header.
///
/// # Arguments
/// * `compressed_data`: The compressed data as a byte slice (including the size header).
///
/// # Returns
/// * `Ok(String)` containing the decompressed string if successful.
/// * `Err(&str)` with an error message if decompression fails or output is invalid UTF-8.
///
/// # Safety
/// This function wraps unsafe FFI calls. It handles memory management
/// for the data returned by the C function and validates UTF-8.
pub fn decompress_rust_data(compressed_data: &[u8]) -> Result<String, &'static str> {
    // Early validation for obviously invalid input to reduce noise during fuzzing
    if compressed_data.is_empty() {
        return Err("Empty input data");
    }
    
    if compressed_data.len() == 1 {
        return Err("Input too small for valid compressed data");
    }

    // Call the C function
    // This is an unsafe block because we are calling C code and dealing with raw pointers.
    let decompressed_c_data = unsafe {
        decompress_data(
            compressed_data.as_ptr() as *const c_char,
            compressed_data.len() as c_ulong,
        )
    };

    // Check if the C function returned a valid buffer
    if decompressed_c_data.buffer.is_null() {
        return Err("Decompression failed");
    }

    // Convert the C data (raw pointer and length) to a Rust Vec<u8>
    // This is also unsafe because we are dereferencing a raw pointer from C.
    let rust_vec: Vec<u8> = unsafe {
        // Create a slice from the raw parts
        let slice = slice::from_raw_parts(
            decompressed_c_data.buffer as *const u8,
            decompressed_c_data.length as usize,
        );
        // Clone the data into a new Vec<u8>
        slice.to_vec()
    };

    // Free the memory allocated by the C function
    // This is crucial to prevent memory leaks.
    unsafe {
        free_decompressed_data(decompressed_c_data);
    }

    // Convert Vec<u8> to String, ensuring valid UTF-8
    match String::from_utf8(rust_vec) {
        Ok(s) => Ok(s),
        Err(_) => Err("Decompressed data is not valid UTF-8"),
    }
}

/// Encodes a value using variable-byte encoding.
///
/// # Arguments
/// * `value`: The value to encode.
///
/// # Returns
/// * `Ok(Vec<u8>)` containing the encoded bytes if successful.
/// * `Err(&str)` with an error message if encoding fails.
///
/// # Safety
/// This function wraps unsafe FFI calls but handles buffer allocation safely.
pub fn encode_varint_rust(value: u64) -> Result<Vec<u8>, &'static str> {
    // Allocate buffer for varint (maximum 10 bytes for 64-bit value)
    let mut buffer = vec![0u8; 10];
    
    let bytes_written = unsafe {
        encode_varint(value as c_ulong, buffer.as_mut_ptr() as *mut c_char)
    };
    
    if bytes_written < 0 || bytes_written > 10 {
        return Err("Invalid bytes written by encode_varint");
    }
    
    buffer.truncate(bytes_written as usize);
    Ok(buffer)
}

/// Decodes a variable-byte encoded value.
///
/// # Arguments
/// * `data`: The encoded data as a byte slice.
///
/// # Returns
/// * `Ok((value, bytes_read))` containing the decoded value and number of bytes consumed if successful.
/// * `Err(&str)` with an error message if decoding fails.
///
/// # Safety
/// This function wraps unsafe FFI calls but handles pointer safety.
pub fn decode_varint_rust(data: &[u8]) -> Result<(u64, usize), &'static str> {
    if data.is_empty() {
        return Err("Empty input data");
    }
    
    let mut value: c_ulong = 0;
    
    let bytes_read = unsafe {
        decode_varint(
            data.as_ptr() as *const c_char,
            data.len() as i32,
            &mut value as *mut c_ulong,
        )
    };
    
    if bytes_read < 0 {
        return Err("Failed to decode varint");
    }
    
    if bytes_read > data.len() as i32 {
        return Err("Invalid bytes read count");
    }
    
    Ok((value as u64, bytes_read as usize))
}

/// Compresses a string using the C library's `compress_string_lz4` function.
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
pub fn compress_rust_string_lz4(s: &str) -> Result<Vec<u8>, &'static str> {
    // Convert the Rust string to a C-compatible string (null-terminated)
    // LZ4 itself doesn't require null termination for the input buffer length,
    // but CString is a convenient way to manage the *const c_char lifetime.
    // We will pass s.len() as the length.
    let c_input_string = match CString::new(s) {
        Ok(cs) => cs,
        Err(_) => return Err("Failed to create CString, input might contain null bytes"),
    };

    let input_ptr = c_input_string.as_ptr();
    // Length of the string (original length, not including CString's null terminator)
    let input_len = s.len() as c_ulong;

    // Call the C function
    let compressed_c_data = unsafe { compress_string_lz4(input_ptr, input_len) };

    if compressed_c_data.buffer.is_null() {
        return Err("LZ4 Compression failed in C library (null buffer returned)");
    }

    let rust_vec: Vec<u8> = unsafe {
        let slice = slice::from_raw_parts(compressed_c_data.buffer as *const u8, compressed_c_data.length as usize);
        slice.to_vec()
    };

    unsafe {
        free_compressed_data(compressed_c_data); // Reuse the existing free function
    }

    Ok(rust_vec)
}

/// Decompresses data using the C library's `decompress_data_lz4` function.
/// The original size is automatically read from the compressed data header.
///
/// # Arguments
/// * `compressed_data`: The compressed data as a byte slice (including the size header).
///
/// # Returns
/// * `Ok(String)` containing the decompressed string if successful.
/// * `Err(&str)` with an error message if decompression fails or output is invalid UTF-8.
///
/// # Safety
/// This function wraps unsafe FFI calls. It handles memory management
/// for the data returned by the C function and validates UTF-8.
pub fn decompress_rust_data_lz4(compressed_data: &[u8]) -> Result<String, &'static str> {
    if compressed_data.is_empty() {
        return Err("Empty input data for LZ4 decompression");
    }
    
    // LZ4 decompression needs at least a header and some data.
    // A single byte varint for original_len=0 plus LZ4 overhead.
    // Smallest valid LZ4 stream is typically a few bytes.
    if compressed_data.len() < 2 { // Minimum: 1 byte varint + 1 byte data (highly unlikely for LZ4)
        return Err("Input too small for valid LZ4 compressed data");
    }

    let decompressed_c_data = unsafe {
        decompress_data_lz4(
            compressed_data.as_ptr() as *const c_char,
            compressed_data.len() as c_ulong,
        )
    };

    if decompressed_c_data.buffer.is_null() {
        return Err("LZ4 Decompression failed in C library (null buffer returned)");
    }

    let rust_vec: Vec<u8> = unsafe {
        let slice = slice::from_raw_parts(
            decompressed_c_data.buffer as *const u8,
            decompressed_c_data.length as usize,
        );
        slice.to_vec()
    };

    unsafe {
        free_decompressed_data(decompressed_c_data); // Reuse the existing free function
    }

    match String::from_utf8(rust_vec) {
        Ok(s) => Ok(s),
        Err(_) => Err("LZ4 Decompressed data is not valid UTF-8"),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_basic() {
        let original_data = "This is a test string for zlib compression, hopefully it gets smaller. then smal file";
        println!("Original data: '{}'", original_data);
        println!("Original length: {}", original_data.len());

        match compress_rust_string(original_data) {
            Ok(compressed_data) => {
                println!("Compressed length: {}", compressed_data.len());
                // For short strings, compression + variable-byte header might not reduce size.
                // The compressed data includes a variable-byte header (1-5 bytes) with the original length.
                assert!(compressed_data.len() > 1, "Compressed data should contain header + compressed content.");
                
                // For longer strings, compression should still be effective despite the header overhead
                if original_data.len() > 200 { // Only assert smaller for much longer strings
                    assert!(compressed_data.len() < original_data.len(), "Compressed data should be smaller than original for large input.");
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

    #[test]
    fn test_compression_decompression_round_trip() {
        let original_data = "This is a test string for zlib compression and decompression round trip test.";
        println!("Original data: '{}'", original_data);
        println!("Original length: {}", original_data.len());

        // Compress the data
        let compressed_data = match compress_rust_string(original_data) {
            Ok(data) => {
                println!("Compressed length: {}", data.len());
                data
            }
            Err(e) => {
                panic!("Compression failed: {}", e);
            }
        };

        // Decompress the data
        match decompress_rust_data(&compressed_data) {
            Ok(decompressed_string) => {
                println!("Decompressed data: '{}'", decompressed_string);
                println!("Decompressed length: {}", decompressed_string.len());
                assert_eq!(original_data, decompressed_string, "Round trip should preserve the original data");
            }
            Err(e) => {
                panic!("Decompression failed: {}", e);
            }
        }
    }

    #[test]
    fn test_decompression_empty_string() {
        let original_data = "";
        
        // Compress empty string
        let compressed_data = compress_rust_string(original_data).expect("Empty string compression should work");
        
        // Decompress it back
        match decompress_rust_data(&compressed_data) {
            Ok(decompressed_string) => {
                assert_eq!(original_data, decompressed_string, "Empty string round trip should work");
            }
            Err(e) => {
                panic!("Decompression of empty string failed: {}", e);
            }
        }
    }

    #[test]
    fn test_decompression_unicode_strings() {
        let test_cases = vec![
            "Hello, 世界!",
            "🦀 Rust FFI 🦀",
            "café naïve résumé",
            "𝕳𝖊𝖑𝖑𝖔",
        ];

        for original_data in test_cases {
            println!("Testing Unicode string: '{}'", original_data);
            
            // Compress the data
            let compressed_data = compress_rust_string(original_data)
                .expect("Unicode string compression should work");
            
            // Decompress the data
            let decompressed_string = decompress_rust_data(&compressed_data)
                .expect("Unicode string decompression should work");
            
            assert_eq!(original_data, decompressed_string, "Unicode round trip should preserve the original data");
        }
    }

    #[test]
    fn test_decompression_with_corrupted_header() {
        let original_data = "This is a test string for testing corrupted header.";
        
        // Compress the data
        let mut compressed_data = compress_rust_string(original_data)
            .expect("Compression should work");
        
        // Corrupt the header (first 8 bytes contain the original length)
        if compressed_data.len() >= 8 {
            compressed_data[0] = 0xFF; // Corrupt first byte of header
            compressed_data[1] = 0xFF; // Corrupt second byte of header
        }
        
        // Try to decompress with corrupted header
        let result = decompress_rust_data(&compressed_data);
        assert!(result.is_err(), "Decompression with corrupted header should fail");
    }

    #[test]
    fn test_variable_byte_encoding_efficiency() {
        // Test different string lengths to verify varint header efficiency
        let test_cases = vec![
            (10, 1),    // Small string: 1-byte varint
            (127, 1),   // Max 1-byte varint
            (128, 2),   // Min 2-byte varint  
            (255, 2),   // Still 2-byte varint
            (16383, 2), // Max 2-byte varint
        ];
        
        for (length, expected_header_bytes) in test_cases {
            let test_string = "A".repeat(length);
            println!("Testing length {} (expecting {}-byte header)", length, expected_header_bytes);
            
            let compressed = compress_rust_string(&test_string)
                .expect("Compression should work");
            
            // Calculate actual header size by comparing with zlib-only compression
            // We can estimate this by checking if the compressed size is reasonable
            let min_expected_size = expected_header_bytes + 8; // varint + minimal zlib output
            assert!(compressed.len() >= min_expected_size, 
                "Compressed size {} should be at least {} (header + minimal zlib)", 
                compressed.len(), min_expected_size);
            
            // Verify round-trip works
            let decompressed = decompress_rust_data(&compressed)
                .expect("Decompression should work");
            assert_eq!(test_string, decompressed, "Round trip should preserve data");
            
            // For highly repetitive strings, compression should be very effective
            if length >= 100 {
                assert!(compressed.len() < length / 2, 
                    "Repetitive string of length {} should compress to less than half size, got {}", 
                    length, compressed.len());
            }
        }
    }

    #[test]
    fn test_decompression_invalid_data() {
        let invalid_compressed_data = vec![0x78, 0x9c, 0xff, 0xff, 0xff]; // Invalid zlib data
        
        let result = decompress_rust_data(&invalid_compressed_data);
        assert!(result.is_err(), "Decompression of invalid data should fail");
    }

    #[test]
    fn test_varint_encoding_basic() {
        let test_cases = vec![
            (0, vec![0x00]),
            (1, vec![0x01]),
            (127, vec![0x7F]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xFF, 0x01]),
            (256, vec![0x80, 0x02]),
            (16383, vec![0xFF, 0x7F]),
            (16384, vec![0x80, 0x80, 0x01]),
        ];

        for (value, expected) in test_cases {
            let encoded = encode_varint_rust(value).expect("Encoding should work");
            assert_eq!(encoded, expected, "Encoding of {} should produce {:?}, got {:?}", value, expected, encoded);
        }
    }

    #[test]
    fn test_varint_decoding_basic() {
        let test_cases = vec![
            (vec![0x00], 0, 1),
            (vec![0x01], 1, 1),
            (vec![0x7F], 127, 1),
            (vec![0x80, 0x01], 128, 2),
            (vec![0xFF, 0x01], 255, 2),
            (vec![0x80, 0x02], 256, 2),
            (vec![0xFF, 0x7F], 16383, 2),
            (vec![0x80, 0x80, 0x01], 16384, 3),
        ];

        for (data, expected_value, expected_bytes_read) in test_cases {
            let (value, bytes_read) = decode_varint_rust(&data).expect("Decoding should work");
            assert_eq!(value, expected_value, "Decoding {:?} should produce value {}, got {}", data, expected_value, value);
            assert_eq!(bytes_read, expected_bytes_read, "Decoding {:?} should read {} bytes, got {}", data, expected_bytes_read, bytes_read);
        }
    }

    #[test]
    fn test_varint_round_trip() {
        let test_values = vec![
            0, 1, 127, 128, 255, 256, 16383, 16384, 65535, 65536,
            1 << 20, 1 << 30, u64::MAX,
        ];

        for value in test_values {
            let encoded = encode_varint_rust(value).expect("Encoding should work");
            let (decoded_value, bytes_read) = decode_varint_rust(&encoded).expect("Decoding should work");
            
            assert_eq!(value, decoded_value, "Round trip should preserve value {}", value);
            assert_eq!(bytes_read, encoded.len(), "Should read all encoded bytes");
        }
    }

    #[test]
    fn test_varint_decode_empty_input() {
        let result = decode_varint_rust(&[]);
        assert!(result.is_err(), "Decoding empty input should fail");
    }

    #[test]
    fn test_varint_decode_incomplete() {
        // Incomplete varint (has continuation bit but no next byte)
        let incomplete_data = vec![0x80];
        let result = decode_varint_rust(&incomplete_data);
        assert!(result.is_err(), "Decoding incomplete varint should fail");
    }

    #[test]
    fn test_varint_decode_with_extra_data() {
        // Varint followed by extra data
        let data = vec![0x01, 0x42, 0x43]; // varint(1) + extra bytes
        let (value, bytes_read) = decode_varint_rust(&data).expect("Should decode the varint part");
        
        assert_eq!(value, 1, "Should decode the varint correctly");
        assert_eq!(bytes_read, 1, "Should only read the varint bytes");
    }

    // Property-based tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use arbitrary::Arbitrary;

        #[derive(Debug, Clone, Arbitrary)]
        struct TestInput {
            data: String,
        }

        // Helper function to run property tests
        fn test_compression_properties(input: TestInput) {
            let result = compress_rust_string(&input.data);
            
            match result {
                Ok(compressed) => {
                    // Property: compressed data should not be empty (zlib always produces some output)
                    assert!(!compressed.is_empty(), "Compressed data should not be empty");
                    
                    // Property: compression should be deterministic
                    let result2 = compress_rust_string(&input.data).unwrap();
                    assert_eq!(compressed, result2, "Compression should be deterministic");
                    
                    // Property: compressed data should be valid (no null pointers, reasonable size)
                    // With variable-byte header (1-5 bytes), compressed size should be reasonable
                    assert!(compressed.len() < input.data.len() + 1005, "Compressed size should be reasonable (original + varint header + overhead)");
                    
                    // Property: round-trip compression/decompression should preserve data
                    match decompress_rust_data(&compressed) {
                        Ok(decompressed) => {
                            assert_eq!(input.data, decompressed, "Round trip should preserve original data");
                        }
                        Err(e) => {
                            panic!("Decompression failed for input '{}': {}", input.data, e);
                        }
                    }
                }
                Err(e) => {
                    // The only expected error is for strings with null bytes
                    assert!(input.data.contains('\0'), "Error should only occur for strings with null bytes, got: {}", e);
                }
            }
        }

        #[test]
        fn test_property_small_strings() {
            let test_cases = vec![
                TestInput { data: "".to_string() },
                TestInput { data: "a".to_string() },
                TestInput { data: "hello".to_string() },
                TestInput { data: "The quick brown fox".to_string() },
            ];
            
            for test_case in test_cases {
                test_compression_properties(test_case);
            }
        }

        #[test]
        fn test_property_repetitive_strings() {
            let test_cases = vec![
                TestInput { data: "a".repeat(100) },
                TestInput { data: "ab".repeat(50) },
                TestInput { data: "hello world ".repeat(10) },
            ];
            
            for test_case in test_cases {
                test_compression_properties(test_case);
            }
        }

        #[test]
        fn test_property_unicode_strings() {
            let test_cases = vec![
                TestInput { data: "Hello, 世界!".to_string() },
                TestInput { data: "🦀 Rust FFI 🦀".to_string() },
                TestInput { data: "café naïve résumé".to_string() },
                TestInput { data: "𝕳𝖊𝖑𝖑𝖔".to_string() },
            ];
            
            for test_case in test_cases {
                test_compression_properties(test_case);
            }
        }

        #[test]
        fn test_property_special_characters() {
            let test_cases = vec![
                TestInput { data: "\n\r\t".to_string() },
                TestInput { data: "!@#$%^&*()".to_string() },
                TestInput { data: "\"'\\`".to_string() },
            ];
            
            for test_case in test_cases {
                test_compression_properties(test_case);
            }
        }

        #[test]
        fn test_property_null_byte_handling() {
            // These should all fail gracefully
            let test_cases = vec![
                TestInput { data: "hello\0world".to_string() },
                TestInput { data: "\0".to_string() },
                TestInput { data: "start\0middle\0end".to_string() },
            ];
            
            for test_case in test_cases {
                test_compression_properties(test_case);
            }
        }
    }
}

#[cfg(test)]
mod lz4_tests {
    use super::*;

    #[test]
    fn test_lz4_compression_basic() {
        let original_data = "This is a test string for LZ4 compression, hopefully it gets smaller.";
        println!("LZ4 Original data: '{}'", original_data);
        println!("LZ4 Original length: {}", original_data.len());

        match compress_rust_string_lz4(original_data) {
            Ok(compressed_data) => {
                println!("LZ4 Compressed length: {}", compressed_data.len());
                // LZ4 is generally very effective.
                // The compressed data includes a variable-byte header (1-5 bytes) with the original length.
                assert!(compressed_data.len() > 1, "LZ4 Compressed data should contain header + compressed content.");
                
                // For non-trivial strings, LZ4 should compress.
                if original_data.len() > 20 {
                     // Adding 10 for varint header to be conservative
                    assert!(compressed_data.len() < original_data.len() + 10, "LZ4 Compressed data + header should be smaller than original for reasonably sized input.");
                }
            }
            Err(e) => {
                panic!("test_lz4_compression_basic failed: {}", e);
            }
        }
    }

    #[test]
    fn test_lz4_compression_empty_string() {
        let original_data = "";
        println!("LZ4 Original data: '{}'", original_data);
        println!("LZ4 Original length: {}", original_data.len());

        match compress_rust_string_lz4(original_data) {
            Ok(compressed_data) => {
                println!("LZ4 Compressed length for empty string: {}", compressed_data.len());
                // Compressing an empty string with LZ4 (plus our header) results in a small output.
                // 1 byte for varint(0) + LZ4's minimum for empty (might be 1 byte or more depending on specifics)
                assert!(compressed_data.len() > 0, "LZ4 Compressed empty string should not be empty.");
                assert!(compressed_data.len() < 10, "LZ4 Compressed empty string should be small.");


                // Test round trip for empty string
                match decompress_rust_data_lz4(&compressed_data) {
                    Ok(decompressed_string) => {
                        assert_eq!(original_data, decompressed_string, "LZ4 Empty string round trip should work");
                    }
                    Err(e) => {
                        panic!("LZ4 Decompression of empty string failed: {}", e);
                    }
                }
            }
            Err(e) => {
                panic!("test_lz4_compression_empty_string failed: {}", e);
            }
        }
    }
    
    #[test]
    fn test_lz4_string_with_null_byte_internal() {
        // CString::new will fail for strings with interior null bytes.
        let original_data = "hello\0world_lz4";
        assert!(compress_rust_string_lz4(original_data).is_err(), "LZ4: Should fail for string with internal null byte due to CString conversion.");
    }

    #[test]
    fn test_lz4_compression_decompression_round_trip() {
        let original_data = "This is a test string for LZ4 compression and decompression round trip test. It needs to be reasonably long for LZ4 to show its benefits.";
        println!("LZ4 Original data: '{}'", original_data);
        println!("LZ4 Original length: {}", original_data.len());

        let compressed_data = match compress_rust_string_lz4(original_data) {
            Ok(data) => {
                println!("LZ4 Compressed length: {}", data.len());
                data
            }
            Err(e) => {
                panic!("LZ4 Compression failed: {}", e);
            }
        };

        match decompress_rust_data_lz4(&compressed_data) {
            Ok(decompressed_string) => {
                println!("LZ4 Decompressed data: '{}'", decompressed_string);
                println!("LZ4 Decompressed length: {}", decompressed_string.len());
                assert_eq!(original_data, decompressed_string, "LZ4 Round trip should preserve the original data");
            }
            Err(e) => {
                panic!("LZ4 Decompression failed: {}", e);
            }
        }
    }

    #[test]
    fn test_lz4_decompression_empty_string_round_trip() {
        // This is also covered by test_lz4_compression_empty_string, but good to have a dedicated one.
        let original_data = "";
        
        let compressed_data = compress_rust_string_lz4(original_data).expect("LZ4 Empty string compression should work");
        
        match decompress_rust_data_lz4(&compressed_data) {
            Ok(decompressed_string) => {
                assert_eq!(original_data, decompressed_string, "LZ4 Empty string round trip should work");
            }
            Err(e) => {
                panic!("LZ4 Decompression of empty string failed: {}", e);
            }
        }
    }

    #[test]
    fn test_lz4_decompression_unicode_strings() {
        let test_cases = vec![
            "Hello, 世界! (LZ4)",
            "🦀 Rust FFI 🦀 (LZ4)",
            "café naïve résumé (LZ4)",
            "𝕳𝖊𝖑𝖑𝖔 (LZ4)",
            "Алло, мир! (LZ4)", // Cyrillic
        ];

        for original_data in test_cases {
            println!("Testing LZ4 Unicode string: '{}'", original_data);
            
            let compressed_data = compress_rust_string_lz4(original_data)
                .unwrap_or_else(|e| panic!("LZ4 Unicode string compression failed for '{}': {}", original_data, e));
            
            let decompressed_string = decompress_rust_data_lz4(&compressed_data)
                .unwrap_or_else(|e| panic!("LZ4 Unicode string decompression failed for '{}': {}", original_data, e));
            
            assert_eq!(original_data, decompressed_string, "LZ4 Unicode round trip should preserve the original data for '{}'", original_data);
        }
    }

    #[test]
    fn test_lz4_decompression_with_corrupted_header() {
        let original_data = "This is a test string for testing corrupted LZ4 header.";
        
        let mut compressed_data = compress_rust_string_lz4(original_data)
            .expect("LZ4 Compression should work");
        
        // Corrupt the varint header.
        // Assuming header is small (e.g., 1-2 bytes for this string length).
        if !compressed_data.is_empty() {
            compressed_data[0] = 0xFF; // Try to make it an invalid varint or point to a huge length
            if compressed_data.len() > 1 {
                 compressed_data[1] = 0xFF;
            }
        } else {
            // If somehow compressed_data is empty (it shouldn't be), this test is moot.
            // But let's make it fail if that's the case, as it indicates an issue in compression.
            panic!("LZ4 compressed data was empty, cannot corrupt header.");
        }
        
        let result = decompress_rust_data_lz4(&compressed_data);
        assert!(result.is_err(), "LZ4 Decompression with corrupted varint header should fail. Got: {:?}", result);
    }

    #[test]
    fn test_lz4_decompression_invalid_data_too_short() {
        // Data that's too short to be valid LZ4 (even after a valid header)
        // 1. Encode a valid header for a small original length (e.g., 10 bytes)
        let original_len: u64 = 10;
        let header = encode_varint_rust(original_len).unwrap();
        
        // 2. Append insufficient or garbage LZ4 data
        let mut invalid_data = header;
        invalid_data.push(0x01); // Not enough data for LZ4_decompress_safe
                                 // for an original length of 10

        let result = decompress_rust_data_lz4(&invalid_data);
        assert!(result.is_err(), "LZ4 Decompression with too short data body should fail. Got: {:?}", result);

        // Test with completely empty data body after header
        let header_only = encode_varint_rust(original_len).unwrap();
        let result_header_only = decompress_rust_data_lz4(&header_only);
         assert!(result_header_only.is_err(), "LZ4 Decompression with only header and no data should fail. Got: {:?}", result_header_only);


        // Test with just a few random bytes that are unlikely to be valid
        let random_bytes = vec![0x12, 0x34, 0x56]; // No valid header, just garbage
        let result_random = decompress_rust_data_lz4(&random_bytes);
        assert!(result_random.is_err(), "LZ4 Decompression of random garbage bytes should fail. Got: {:?}", result_random);

        // Test with data that is too short to even contain a minimal header
        let too_short_for_header = vec![];
        let result_too_short_header = decompress_rust_data_lz4(&too_short_for_header);
        assert!(result_too_short_header.is_err(), "LZ4 Decompression of empty byte slice should fail. Got: {:?}", result_too_short_header);
    }

     #[test]
    fn test_lz4_highly_compressible_data() {
        let original_data = "a".repeat(10000); // Highly compressible
        
        let compressed_data = compress_rust_string_lz4(&original_data)
            .expect("LZ4 compression of repetitive data should work");
        
        println!("LZ4 Highly compressible: Original size: {}, Compressed size: {}", original_data.len(), compressed_data.len());
        // header (max 10 bytes for 10000) + LZ4 compressed data.
        // LZ4 should achieve very high compression for this.
        assert!(compressed_data.len() < original_data.len() / 10 + 10, "LZ4 should compress repetitive data significantly.");

        let decompressed_string = decompress_rust_data_lz4(&compressed_data)
            .expect("LZ4 decompression of repetitive data should work");
        
        assert_eq!(original_data, decompressed_string, "LZ4 Round trip for repetitive data should preserve the original data");
    }

    #[test]
    fn test_lz4_random_like_data() {
        // More random-like, less compressible data
        // (Still text, so somewhat compressible, but less than "aaaa...")
        let original_data = "TheV0yage0fTheBeagleByCharlesDarwinChapterI.";
        
        let compressed_data = compress_rust_string_lz4(original_data)
            .expect("LZ4 compression of less compressible data should work");

        println!("LZ4 Less compressible: Original size: {}, Compressed size: {}", original_data.len(), compressed_data.len());
        // For less compressible data, the gain might be smaller or even negative if string is short,
        // due to header and LZ4 minimums.
        if original_data.len() > 50 { // Arbitrary threshold for expecting some compression
             // Adding 10 for varint header
            assert!(compressed_data.len() < original_data.len() + 10, "LZ4 should not expand data significantly for moderate strings.");
        }

        let decompressed_string = decompress_rust_data_lz4(&compressed_data)
            .expect("LZ4 decompression of less compressible data should work");
        
        assert_eq!(original_data, decompressed_string, "LZ4 Round trip for less compressible data should preserve the original data");
    }
}

#[cfg(test)]
mod reproduce_fuzzing_bug {
    use super::*;

    #[test]
    fn test_reproduce_original_crash() {
        // This is the exact input that caused the MemorySanitizer crash
        // Base64: eDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDQ0NDRt6Ojo6Ojo6Ojo6Ojo6Ojo6Ojo6Ojo6AAAAADo6Ojo6Ojo6Ojo6Ojo7Ojo6Ojo6Ojo6OgAAAAAAAAAAAEAAAAAB+jo6Ojo6OjoDj8=
        let crash_input: Vec<u8> = vec![
            0x78, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34,
            0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34,
            0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34,
            0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34,
            0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34, 0x34,
            0x6d, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8,
            0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0x0, 0x0, 0x0, 0x0, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8,
            0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xec, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8,
            0xe8, 0xe8, 0xe8, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x7,
            0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe8, 0xe, 0x3f,
        ];

        // This should not panic or cause a MemorySanitizer error
        // The function should gracefully handle invalid input
        match decompress_rust_data_lz4(&crash_input) {
            Ok(_) => {
                // If decompression succeeds, that's fine too
                println!("Decompression succeeded (unexpected but valid)");
            }
            Err(e) => {
                // Expected: decompression should fail gracefully for invalid input
                println!("Decompression failed as expected: {}", e);
            }
        }
    }
}
