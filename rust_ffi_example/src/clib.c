#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <zlib.h>
#include <lz4.h>

// Define a struct to return both buffer and length
typedef struct {
    char *buffer;
    unsigned long length;
} CompressedData;

// Define a struct to return decompressed data
typedef struct {
    char *buffer;
    unsigned long length;
} DecompressedData;

// Variable-byte encoding functions

// Encode a length as variable-byte encoding
// Returns the number of bytes written
int encode_varint(unsigned long value, char *buffer) {
    int bytes_written = 0;
    while (value >= 0x80) {
        buffer[bytes_written++] = (char)((value & 0x7F) | 0x80);
        value >>= 7;
    }
    buffer[bytes_written++] = (char)(value & 0x7F);
    return bytes_written;
}

// Decode a variable-byte encoded length
// Returns the number of bytes read, or -1 on error
int decode_varint(const char *buffer, int max_bytes, unsigned long *value) {
    *value = 0;
    int shift = 0;
    int bytes_read = 0;
    
    while (bytes_read < max_bytes) {
        unsigned char byte = (unsigned char)buffer[bytes_read++];
        *value |= (unsigned long)(byte & 0x7F) << shift;
        
        if ((byte & 0x80) == 0) {
            return bytes_read; // Success
        }
        
        shift += 7;
        if (shift >= 64) {
            // Reduce noise during fuzzing - only print in debug mode
            #ifdef DEBUG_FUZZING
            fprintf(stderr, "Varint overflow: length too large\n");
            #endif
            return -1; // Overflow
        }
    }
    
    // Reduce noise during fuzzing - only print in debug mode
    #ifdef DEBUG_FUZZING
    fprintf(stderr, "Incomplete varint: unexpected end of data\n");
    #endif
    return -1; // Incomplete varint
}

// Function to compress a string using zlib with variable-byte length header
// The compressed data format: [varint original length][zlib compressed data]
// The caller is responsible for freeing the returned buffer
CompressedData compress_string(const char *input, unsigned long input_len) {
    unsigned long compressed_bound = compressBound(input_len);
    // Allocate buffer for: max 5-byte varint header + compressed data
    unsigned long total_buffer_size = 5 + compressed_bound;
    char *output_buffer = (char *)malloc(total_buffer_size);
    CompressedData result = {NULL, 0};

    if (output_buffer == NULL) {
        perror("Failed to allocate memory for compression");
        return result; // Return empty result
    }

    // Encode original length as varint at the beginning
    int header_size = encode_varint(input_len, output_buffer);

    // Compress data after the varint header
    unsigned long compressed_len = compressed_bound;
    int res = compress((Bytef *)(output_buffer + header_size), &compressed_len, (const Bytef *)input, input_len);

    if (res != Z_OK) {
        fprintf(stderr, "Compression failed: %d\n", res);
        free(output_buffer);
        return result; // Return empty result
    }

    result.buffer = output_buffer;
    result.length = header_size + compressed_len; // Header + compressed data
    return result;
}

// Function to decompress data using zlib, automatically reading original size from varint header
// Expects input format: [varint original length][zlib compressed data]
// The caller is responsible for freeing the returned buffer
DecompressedData decompress_data(const char *input, unsigned long input_len) {
    DecompressedData result = {NULL, 0};
    
    // Check minimum input size (at least 1 byte for varint + some compressed data)
    if (input_len < 2) {
        // Reduce noise during fuzzing - only print in debug mode
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Invalid compressed data: too small (need at least 2 bytes)\n");
        #endif
        return result;
    }
    
    // Decode original length from varint header
    unsigned long original_len;
    int header_size = decode_varint(input, input_len, &original_len);
    if (header_size < 0) {
        // Reduce noise during fuzzing - only print in debug mode
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Invalid compressed data: failed to decode varint header\n");
        #endif
        return result;
    }
    
    // Check that we have enough data after the header
    if ((unsigned long)header_size >= input_len) {
        // Reduce noise during fuzzing - only print in debug mode
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Invalid compressed data: no data after varint header\n");
        #endif
        return result;
    }
    
    // Sanity check on original length (prevent absurdly large allocations)
    if (original_len > 100 * 1024 * 1024) { // 100MB limit
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Invalid compressed data: original length too large (%lu bytes)\n", original_len);
        #endif
        return result;
    }
    
    // Allocate buffer for decompressed data
    char *output_buffer = (char *)malloc(original_len + 1); // +1 for potential null terminator
    if (output_buffer == NULL) {
        perror("Failed to allocate memory for decompression");
        return result;
    }

    // Decompress data (skip the varint header)
    unsigned long actual_output_len = original_len;
    int res = uncompress((Bytef *)output_buffer, &actual_output_len, 
                        (const Bytef *)(input + header_size), input_len - header_size);

    if (res != Z_OK) {
        // Reduce noise during fuzzing - only print in debug mode
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Decompression failed: %d\n", res);
        #endif
        free(output_buffer);
        return result;
    }
    
    // Verify that decompressed length matches expected length
    if (actual_output_len != original_len) {
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Decompression length mismatch: expected %lu, got %lu\n", 
                original_len, actual_output_len);
        #endif
        free(output_buffer);
        return result;
    }

    result.buffer = output_buffer;
    result.length = actual_output_len;
    return result;
}

// Function to compress a string using LZ4 with variable-byte length header
// The compressed data format: [varint original length][LZ4 compressed data]
// The caller is responsible for freeing the returned buffer
CompressedData compress_string_lz4(const char *input, unsigned long input_len) {
    // Calculate the maximum compressed size using LZ4_compressBound
    int lz4_max_compressed_size = LZ4_compressBound((int)input_len);
    if (lz4_max_compressed_size <= 0) {
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "LZ4_compressBound failed or input size is 0.\n");
        #endif
        return (CompressedData){NULL, 0};
    }

    // Allocate buffer for: max 5-byte varint header + compressed data
    unsigned long total_buffer_size = 5 + (unsigned long)lz4_max_compressed_size;
    char *output_buffer = (char *)malloc(total_buffer_size);
    CompressedData result = {NULL, 0};

    if (output_buffer == NULL) {
        perror("Failed to allocate memory for LZ4 compression");
        return result; // Return empty result
    }

    // Encode original length as varint at the beginning
    int header_size = encode_varint(input_len, output_buffer);

    // Compress data after the varint header
    int compressed_data_size = LZ4_compress_default(input, output_buffer + header_size, (int)input_len, (int)(total_buffer_size - header_size));

    if (compressed_data_size <= 0) {
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "LZ4_compress_default failed: %d\n", compressed_data_size);
        #endif
        free(output_buffer);
        return result; // Return empty result
    }

    result.buffer = output_buffer;
    result.length = header_size + compressed_data_size; // Header + compressed data
    return result;
}

// Function to decompress data using LZ4, automatically reading original size from varint header
// Expects input format: [varint original length][LZ4 compressed data]
// The caller is responsible for freeing the returned buffer
DecompressedData decompress_data_lz4(const char *input, unsigned long input_len) {
    DecompressedData result = {NULL, 0};

    // Check minimum input size (at least 1 byte for varint + some compressed data)
    if (input_len < 2) {
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Invalid LZ4 compressed data: too small (need at least 2 bytes)\n");
        #endif
        return result;
    }

    // Decode original length from varint header
    unsigned long original_len;
    int header_size = decode_varint(input, input_len, &original_len);
    if (header_size < 0) {
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Invalid LZ4 compressed data: failed to decode varint header\n");
        #endif
        return result;
    }

    // Check that we have enough data after the header
    if ((unsigned long)header_size >= input_len) {
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Invalid LZ4 compressed data: no data after varint header\n");
        #endif
        return result;
    }

    // Sanity check on original length (prevent absurdly large allocations)
    // Using the same limit as zlib version for consistency
    if (original_len > 100 * 1024 * 1024) { // 100MB limit
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "Invalid LZ4 compressed data: original length too large (%lu bytes)\n", original_len);
        #endif
        return result;
    }
    
    if (original_len == 0) { // Handle zero-length original string case
        char *output_buffer = (char *)malloc(1); // Allocate 1 byte for empty string
        if (output_buffer == NULL) {
            perror("Failed to allocate memory for LZ4 decompression (empty string)");
            return result;
        }
        output_buffer[0] = '\0';
        result.buffer = output_buffer;
        result.length = 0;
        return result;
    }


    // Allocate buffer for decompressed data
    char *output_buffer = (char *)malloc(original_len + 1); // +1 for potential null terminator
    if (output_buffer == NULL) {
        perror("Failed to allocate memory for LZ4 decompression");
        return result;
    }

    // Decompress data (skip the varint header)
    int decompressed_size = LZ4_decompress_safe(input + header_size, output_buffer, (int)(input_len - header_size), (int)original_len);

    if (decompressed_size < 0) {
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "LZ4_decompress_safe failed: %d\n", decompressed_size);
        #endif
        free(output_buffer);
        return result;
    }

    // Verify that decompressed length matches expected length
    if ((unsigned long)decompressed_size != original_len) {
        #ifdef DEBUG_FUZZING
        fprintf(stderr, "LZ4 Decompression length mismatch: expected %lu, got %d\n",
                original_len, decompressed_size);
        #endif
        free(output_buffer);
        return result;
    }
    
    output_buffer[original_len] = '\0'; // Ensure null termination

    result.buffer = output_buffer;
    result.length = (unsigned long)decompressed_size;
    return result;
}

// Function to free the memory allocated by compress_string
void free_compressed_data(CompressedData data) {
    if (data.buffer != NULL) {
        free(data.buffer);
    }
}

// Function to free the memory allocated by decompress_data
void free_decompressed_data(DecompressedData data) {
    if (data.buffer != NULL) {
        free(data.buffer);
    }
}

#ifdef BUILD_TEST_MAIN
// Main function for testing the C code directly (optional)
int main() {
    const char *test_str = "Hello, world! This is a test string for zlib compression.";
    unsigned long test_str_len = strlen(test_str);

    printf("Original string: '%s'\n", test_str);
    printf("Original length: %lu\n", test_str_len);

    CompressedData compressed = compress_string(test_str, test_str_len);

    if (compressed.buffer) {
        printf("Successfully compressed string.\n");
        printf("Compressed length: %lu\n", compressed.length);
        // Optional: print compressed data (might be binary)
        // printf("Compressed data (hex): ");
        // for (unsigned long i = 0; i < compressed.length; ++i) {
        //     printf("%02x", (unsigned char)compressed.buffer[i]);
        // }
        // printf("\n");
        free_compressed_data(compressed);
    } else {
        printf("Compression failed.\n");
    }

    return 0;
}
#endif
