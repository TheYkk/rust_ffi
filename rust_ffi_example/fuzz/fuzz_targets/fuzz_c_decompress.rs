#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_ffi_example::{decompress_data, free_decompressed_data};
use std::os::raw::{c_ulong, c_char};

fuzz_target!(|data: &[u8]| {
    let input_ptr = data.as_ptr() as *const c_char;
    let input_len = data.len() as c_ulong;

    // The decompress_data function expects the compressed data to start with
    // a varint-encoded original size. The fuzzer will generate arbitrary data,
    // so this might often be invalid, which is fine as it tests robustness.
    unsafe {
        let decompressed_result = decompress_data(input_ptr, input_len);

        if !decompressed_result.buffer.is_null() {
            // Ensure the decompressed data is freed, regardless of its content or validity.
            free_decompressed_data(decompressed_result);
        }
    }
});
