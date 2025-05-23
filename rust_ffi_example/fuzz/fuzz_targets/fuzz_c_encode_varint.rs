#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_ffi_example::encode_varint; // C FFI function
use std::os::raw::{c_ulong, c_char};

const MAX_VARINT_LENGTH: usize = 10;

fuzz_target!(|value_to_encode: u64| {
    let mut buffer: [u8; MAX_VARINT_LENGTH] = [0; MAX_VARINT_LENGTH];
    
    unsafe {
        // Call the C FFI function
        // int encode_varint(uint64_t value, uint8_t *buffer);
        let _bytes_written = encode_varint(
            value_to_encode as c_ulong,        // value
            buffer.as_mut_ptr() as *mut c_char, // buffer - cast to match c_char type
        );
        // The result (bytes_written) can be optionally checked or used.
        // No explicit freeing is needed for the stack-allocated buffer.
    }
});
