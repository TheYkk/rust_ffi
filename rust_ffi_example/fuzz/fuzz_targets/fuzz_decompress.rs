#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_ffi_example::decompress_rust_data;

fuzz_target!(|data: &[u8]| {
    // Call decompress_rust_data with the fuzzer-provided data.
    // The function returns Result<String, &'static str>.
    // We don't need to explicitly check the result; if it's an Err,
    // that's a valid outcome for malformed input. If it panics,
    // libfuzzer will catch that.
    let _ = decompress_rust_data(data);
});
