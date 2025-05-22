#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_ffi_example::encode_varint_rust;

fuzz_target!(|data: &[u8]| {
    // Convert the fuzzer input to different types of test values
    if !data.is_empty() {
        // Test with direct byte interpretation as u64
        let mut value = 0u64;
        for (i, &byte) in data.iter().enumerate() {
            if i >= 8 { break; } // Limit to 8 bytes for u64
            value |= (byte as u64) << (i * 8);
        }
        
        // Test encoding this value
        let _ = encode_varint_rust(value);
        
        // Test with smaller values derived from the input
        if data.len() >= 1 {
            let _ = encode_varint_rust(data[0] as u64);
        }
        
        if data.len() >= 2 {
            let small_value = ((data[0] as u16) | ((data[1] as u16) << 8)) as u64;
            let _ = encode_varint_rust(small_value);
        }
        
        if data.len() >= 4 {
            let medium_value = ((data[0] as u32) | 
                               ((data[1] as u32) << 8) |
                               ((data[2] as u32) << 16) |
                               ((data[3] as u32) << 24)) as u64;
            let _ = encode_varint_rust(medium_value);
        }
    }
    
    // Test edge cases
    let _ = encode_varint_rust(0);
    let _ = encode_varint_rust(1);
    let _ = encode_varint_rust(127);
    let _ = encode_varint_rust(128);
    let _ = encode_varint_rust(u64::MAX);
    
    // Test values around powers of 2
    for i in 0..64 {
        let power_of_2 = 1u64 << i;
        let _ = encode_varint_rust(power_of_2);
        if power_of_2 > 1 {
            let _ = encode_varint_rust(power_of_2 - 1);
        }
        if power_of_2 < u64::MAX {
            let _ = encode_varint_rust(power_of_2 + 1);
        }
    }
}); 