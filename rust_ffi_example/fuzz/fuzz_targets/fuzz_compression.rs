#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_ffi_example::compress_rust_string;
use std::str;

fuzz_target!(|data: &[u8]| {
    // Try to convert the fuzzer input to a string
    // This covers both valid UTF-8 and invalid byte sequences
    match str::from_utf8(data) {
        Ok(s) => {
            // Test with valid UTF-8 strings
            let _ = compress_rust_string(s);
        }
        Err(_) => {
            // For invalid UTF-8, try to create a lossy string representation
            let lossy_string = String::from_utf8_lossy(data);
            let _ = compress_rust_string(&lossy_string);
        }
    }
    
    // Also test specific edge cases based on the fuzzer data
    if !data.is_empty() {
        // Create various test patterns based on the input
        let len = data.len().min(10000); // Limit size to prevent excessive memory usage
        
        // Test repeated pattern
        if data.len() > 0 {
            let pattern = match str::from_utf8(&data[..1]) {
                Ok(p) => p.repeat(len),
                Err(_) => "A".repeat(len), // Fallback to safe pattern
            };
            let _ = compress_rust_string(&pattern);
        }
        
        // Test string with embedded nulls (this should fail gracefully)
        if data.len() > 2 {
            let mut test_with_null = String::from("Hello");
            test_with_null.push('\0');
            test_with_null.push_str("World");
            // This should return an error due to the null byte
            let _ = compress_rust_string(&test_with_null);
        }
    }
    
    // Test empty string
    let _ = compress_rust_string("");
    
    // Test very long strings (but with reasonable limits)
    if data.len() > 10 {
        let long_string = "x".repeat((data[0] as usize * 100).min(50000));
        let _ = compress_rust_string(&long_string);
    }
});
