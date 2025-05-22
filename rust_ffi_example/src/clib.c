#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <zlib.h>

// Define a struct to return both buffer and length
typedef struct {
    char *buffer;
    unsigned long length;
} CompressedData;

// Function to compress a string using zlib
// The caller is responsible for freeing the returned buffer
CompressedData compress_string(const char *input, unsigned long input_len) {
    unsigned long output_len_bound = compressBound(input_len);
    char *output_buffer = (char *)malloc(output_len_bound);
    CompressedData result = {NULL, 0};

    if (output_buffer == NULL) {
        perror("Failed to allocate memory for compression");
        return result; // Return empty result
    }

    int res = compress((Bytef *)output_buffer, &output_len_bound, (const Bytef *)input, input_len);

    if (res != Z_OK) {
        fprintf(stderr, "Compression failed: %d\n", res);
        free(output_buffer);
        return result; // Return empty result
    }

    result.buffer = output_buffer;
    result.length = output_len_bound;
    return result;
}

// Function to free the memory allocated by compress_string
void free_compressed_data(CompressedData data) {
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
