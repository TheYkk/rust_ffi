#![no_main]
use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use rust_ffi_example::decompress_rust_data_zstd;

// Define a wrapper struct for the input data if needed, or directly use Vec<u8>
#[derive(Debug, Clone)]
struct FuzzInput {
    data: Vec<u8>,
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, libfuzzer_sys::arbitrary::Error> {
        let data = Vec::<u8>::arbitrary(u)?;
        Ok(FuzzInput { data })
    }
}

fuzz_target!(|input: FuzzInput| {
    // Pass the arbitrary byte array to the ZSTD decompression function
    // The function is expected to handle malformed/invalid data gracefully
    // by returning an Err, not by panicking or crashing.
    let _ = decompress_rust_data_zstd(&input.data);
    
    // No assertion is needed on the Ok_or_Err result itself, as the primary goal
    // is to detect panics/crashes during the decompression of potentially invalid data.
    // The C library should have safeguards against buffer overflows, infinite loops, etc.
    // and the Rust wrapper should correctly handle errors returned by the C library.
});
