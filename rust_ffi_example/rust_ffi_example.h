#ifndef RUST_FFI_EXAMPLE_H
#define RUST_FFI_EXAMPLE_H

#include <stdint.h> // For int32_t
#include <stddef.h> // For size_t (though unsigned long is used directly)

// Define structures to match Rust's FFI representation
typedef struct {
    char* buffer;
    unsigned long length;
} CompressedData;

typedef struct {
    char* buffer;
    unsigned long length;
} DecompressedData;

#ifdef __cplusplus
extern "C" {
#endif

// FFI function declarations

/**
 * Compresses a string using the default algorithm.
 * The caller is responsible for freeing the returned CompressedData using free_compressed_data.
 */
CompressedData compress_string(const char* input, unsigned long input_len);

/**
 * Frees the memory allocated for CompressedData.
 */
void free_compressed_data(CompressedData data);

/**
 * Decompresses data using the default algorithm.
 * The caller is responsible for freeing the returned DecompressedData using free_decompressed_data.
 */
DecompressedData decompress_data(const char* input, unsigned long input_len);

/**
 * Frees the memory allocated for DecompressedData.
 */
void free_decompressed_data(DecompressedData data);

/**
 * Compresses a string using the LZ4 algorithm.
 * The caller is responsible for freeing the returned CompressedData using free_compressed_data.
 */
CompressedData compress_string_lz4(const char* input, unsigned long input_len);

/**
 * Decompresses data compressed with LZ4.
 * The caller is responsible for freeing the returned DecompressedData using free_decompressed_data.
 */
DecompressedData decompress_data_lz4(const char* input, unsigned long input_len);

/**
 * Compresses a string using the Zstd algorithm.
 * The caller is responsible for freeing the returned CompressedData using free_compressed_data.
 */
CompressedData compress_string_zstd(const char* input, unsigned long input_len);

/**
 * Decompresses data compressed with Zstd.
 * The caller is responsible for freeing the returned DecompressedData using free_decompressed_data.
 */
DecompressedData decompress_data_zstd(const char* input, unsigned long input_len);

/**
 * Encodes an unsigned long value into a VarInt format.
 * buffer must be large enough to hold the encoded VarInt (max 10 bytes for u64).
 * Returns the number of bytes written to the buffer, or a negative value on error.
 */
int32_t encode_varint(unsigned long value, char* buffer);

/**
 * Decodes a VarInt from the buffer into an unsigned long value.
 * max_bytes indicates the maximum number of bytes to read from the buffer.
 * The decoded value is stored in the `value` output parameter.
 * Returns the number of bytes read from the buffer, or a negative value on error
 * (e.g., if VarInt is malformed or exceeds max_bytes).
 */
int32_t decode_varint(const char* buffer, int32_t max_bytes, unsigned long* value);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // RUST_FFI_EXAMPLE_H
