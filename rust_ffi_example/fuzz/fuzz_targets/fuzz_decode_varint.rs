#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_ffi_example::{decode_varint_rust, encode_varint_rust};

fuzz_target!(|data: &[u8]| {
    // Test decoding the fuzzer input directly
    let _ = decode_varint_rust(data);
    
    // Generate some valid varints and test them
    if !data.is_empty() {
        // Create values from the fuzzer input and encode/decode them for round-trip testing
        let mut value = 0u64;
        for (i, &byte) in data.iter().enumerate() {
            if i >= 8 { break; } // Limit to 8 bytes for u64
            value |= (byte as u64) << (i * 8);
        }
        
        // Test round-trip: encode then decode
        if let Ok(encoded) = encode_varint_rust(value) {
            let _ = decode_varint_rust(&encoded);
            
            // Test with truncated versions of the encoded data
            for len in 1..=encoded.len() {
                let truncated = &encoded[..len];
                let _ = decode_varint_rust(truncated);
            }
            
            // Test with the encoded data plus additional fuzzer bytes
            if data.len() > 8 {
                let mut extended = encoded.clone();
                extended.extend_from_slice(&data[8..]);
                let _ = decode_varint_rust(&extended);
            }
        }
    }
    
    // Test various patterns of bytes
    if data.len() >= 2 {
        // Test all continuation bits set
        let all_continuation = data.iter().map(|&b| b | 0x80).collect::<Vec<u8>>();
        let _ = decode_varint_rust(&all_continuation);
        
        // Test no continuation bits set (except potentially the last byte)
        let no_continuation = data.iter().map(|&b| b & 0x7F).collect::<Vec<u8>>();
        let _ = decode_varint_rust(&no_continuation);
        
        // Test alternating continuation bits
        let alternating = data.iter().enumerate()
            .map(|(i, &b)| if i % 2 == 0 { b | 0x80 } else { b & 0x7F })
            .collect::<Vec<u8>>();
        let _ = decode_varint_rust(&alternating);
    }
    
    // Test edge case patterns
    let edge_cases = vec![
        vec![0x00],                    // minimum value
        vec![0x7F],                    // max 1-byte value  
        vec![0x80, 0x01],             // min 2-byte value
        vec![0xFF, 0x7F],             // max 2-byte value
        vec![0x80, 0x80, 0x01],       // min 3-byte value
        vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01], // max value
        vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80], // overflow case
    ];
    
    for case in edge_cases {
        let _ = decode_varint_rust(&case);
    }
    
    // Test empty input
    let _ = decode_varint_rust(&[]);
    
    // Test single bytes with various bit patterns
    for byte in [0x00, 0x01, 0x7F, 0x80, 0xFF] {
        let _ = decode_varint_rust(&[byte]);
    }
}); 