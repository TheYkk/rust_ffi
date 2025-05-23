#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_ffi_example::decode_varint; // C FFI function
use std::os::raw::{c_ulong, c_char};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        // The decode_varint function might not handle empty slices well,
        // or it might be a valid case to test. If it's expected to handle
        // empty or very short slices gracefully (e.g., by returning 0 bytes read),
        // this check can be removed. For now, let's avoid passing empty data
        // if the C function isn't robust against it.
        return;
    }

    let mut decoded_value: c_ulong = 0;
    
    unsafe {
        // Call the C FFI function
        // int decode_varint(const uint8_t *data, int max_bytes, uint64_t *value_out);
        let _bytes_read = decode_varint(
            data.as_ptr() as *const c_char, // data - cast to match c_char type
            data.len() as i32,              // max_bytes (as per C function signature)
            &mut decoded_value as *mut c_ulong // value_out
        );
        // The result (bytes_read) and decoded_value can be optionally checked or used.
    }
});
