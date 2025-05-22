#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_ffi_example::{encode_varint_rust, decode_varint_rust};

fuzz_target!(|data: &[u8]| {
    if !data.is_empty() {
        // Convert fuzzer input to various u64 values for testing
        let test_values = generate_test_values(data);
        
        for value in test_values {
            // Test the round-trip property: encode(value) -> decode(encoded) should equal value
            if let Ok(encoded) = encode_varint_rust(value) {
                match decode_varint_rust(&encoded) {
                    Ok((decoded_value, bytes_read)) => {
                        // Property 1: Round-trip should preserve the value
                        assert_eq!(value, decoded_value, "Round-trip failed for value {}", value);
                        
                        // Property 2: Should read exactly the encoded length
                        assert_eq!(bytes_read, encoded.len(), "Bytes read mismatch for value {}", value);
                        
                        // Property 3: Encoded length should be reasonable (1-10 bytes for u64)
                        assert!(encoded.len() >= 1 && encoded.len() <= 10, "Invalid encoded length {} for value {}", encoded.len(), value);
                        
                        // Property 4: Test that decoding with extra data works correctly
                        if data.len() > 8 {
                            let mut extended = encoded.clone();
                            extended.extend_from_slice(&data[8..data.len().min(16)]);
                            
                            if let Ok((extended_decoded_value, extended_bytes_read)) = decode_varint_rust(&extended) {
                                assert_eq!(value, extended_decoded_value, "Decoding with extra data failed for value {}", value);
                                assert_eq!(bytes_read, extended_bytes_read, "Bytes read changed with extra data for value {}", value);
                            }
                        }
                        
                        // Property 5: Test deterministic encoding
                        if let Ok(encoded2) = encode_varint_rust(value) {
                            assert_eq!(encoded, encoded2, "Encoding is not deterministic for value {}", value);
                        }
                    }
                    Err(_) => {
                        // If decoding fails, the encoded data should be invalid (this shouldn't happen for valid encode output)
                        panic!("Decoding failed for value {} with encoded data {:?}", value, encoded);
                    }
                }
            }
        }
        
        // Test with invalid/corrupted varint data derived from fuzzer input
        test_invalid_varints(data);
    }
});

fn generate_test_values(data: &[u8]) -> Vec<u64> {
    let mut values = Vec::new();
    
    // Extract various u64 values from the fuzzer input
    if !data.is_empty() {
        // Single byte value
        values.push(data[0] as u64);
        
        // Two-byte value
        if data.len() >= 2 {
            let val = ((data[0] as u16) | ((data[1] as u16) << 8)) as u64;
            values.push(val);
        }
        
        // Four-byte value
        if data.len() >= 4 {
            let val = ((data[0] as u32) | 
                      ((data[1] as u32) << 8) |
                      ((data[2] as u32) << 16) |
                      ((data[3] as u32) << 24)) as u64;
            values.push(val);
        }
        
        // Eight-byte value
        if data.len() >= 8 {
            let mut val = 0u64;
            for i in 0..8 {
                val |= (data[i] as u64) << (i * 8);
            }
            values.push(val);
        }
        
        // Values based on first few bytes interpreted in different ways
        for &byte in data.iter().take(4) {
            values.push(byte as u64);
            values.push((byte as u64) << 7);
            values.push((byte as u64) << 14);
            values.push((byte as u64) << 21);
        }
    }
    
    // Add some deterministic edge cases
    values.extend_from_slice(&[
        0, 1, 127, 128, 255, 256, 16383, 16384, 65535, 65536,
        (1u64 << 20) - 1, 1u64 << 20, (1u64 << 20) + 1,
        (1u64 << 32) - 1, 1u64 << 32, (1u64 << 32) + 1,
        u64::MAX - 1, u64::MAX,
    ]);
    
    values
}

fn test_invalid_varints(data: &[u8]) {
    // Test various corruption patterns
    if data.len() >= 2 {
        // Create potentially invalid varints by setting continuation bits incorrectly
        let mut corrupted = data.to_vec();
        
        // All bytes have continuation bit set (potential overflow)
        for byte in &mut corrupted {
            *byte |= 0x80;
        }
        let _ = decode_varint_rust(&corrupted);
        
        // Remove continuation bits from all but last byte (potential underflow/incorrect format)
        let len = corrupted.len();
        for byte in &mut corrupted[..len-1] {
            *byte |= 0x80; // Set continuation bit
        }
        corrupted[len-1] &= 0x7F; // Clear continuation bit on last byte
        let _ = decode_varint_rust(&corrupted);
    }
    
    // Test with very long sequences that might cause overflow
    if data.len() >= 11 {
        let long_varint = &data[..11]; // More than 10 bytes (max for u64)
        let _ = decode_varint_rust(long_varint);
    }
    
    // Test truncated valid varints
    for value in [128u64, 16384, 1u64 << 32, u64::MAX] {
        if let Ok(encoded) = encode_varint_rust(value) {
            // Test all possible truncations
            for len in 1..encoded.len() {
                let truncated = &encoded[..len];
                let _ = decode_varint_rust(truncated);
            }
        }
    }
} 