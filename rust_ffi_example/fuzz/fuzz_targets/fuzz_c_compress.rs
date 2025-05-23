#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_ffi_example::{compress_string, free_compressed_data};
use std::os::raw::{c_char, c_ulong};

fuzz_target!(|data: &[u8]| {
    // Create a C-compatible string: append a null byte.
    // This is safer than trying to find a null byte in potentially invalid UTF-8 data.
    let mut c_string_vec: Vec<u8> = data.to_vec();
    c_string_vec.push(0);

    let input_ptr = c_string_vec.as_ptr() as *const c_char;
    // The length for compress_string should be the length of the original data,
    // not including the manually appended null terminator.
    let input_len = data.len() as c_ulong;

    unsafe {
        let compressed_result = compress_string(input_ptr, input_len);

        if !compressed_result.buffer.is_null() {
            // Ensure the compressed data is freed, regardless of its content or validity.
            free_compressed_data(compressed_result);
        }
    }
});
