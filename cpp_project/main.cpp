#include <iostream>
#include <vector>
#include <string>
#include <fstream>
#include <iomanip>
#include <cstring> // For strcmp, strlen
#include <algorithm> // For std::min
#include <sstream> // Added for std::stringstream

// For isatty and fileno
#include <unistd.h> // For isatty (on POSIX systems like Linux)
#include <cstdio>   // For fileno (on POSIX systems like Linux)


// Adjust the path based on the final location of the header file
#include "../rust_ffi_example/rust_ffi_example.h"

void print_usage(const char* program_name) {
    std::cerr << "Usage:" << std::endl;
    std::cerr << "  " << program_name << " compress [text]              - Compress text (or from stdin)" << std::endl;
    std::cerr << "  " << program_name << " decompress <file>            - Decompress binary file" << std::endl;
    std::cerr << "  " << program_name << " encode-varint <number>         - Encode a u64 number into varint format (output as hex)" << std::endl;
    std::cerr << "  " << program_name << " decode-varint <hex_bytes>      - Decode varint hex bytes into a u64 number" << std::endl;
    std::cerr << "\nExamples:" << std::endl;
    std::cerr << "  " << program_name << " compress \"hello world\"" << std::endl;
    std::cerr << "  echo \"hello from pipe\" | " << program_name << " compress" << std::endl;
    std::cerr << "  " << program_name << " decompress compressed_output.bin" << std::endl;
    std::cerr << "  " << program_name << " encode-varint 12345" << std::endl;
    std::cerr << "  " << program_name << " decode-varint c96001" << std::endl;
}

// Helper function to convert byte vector to hex string
std::string bytes_to_hex_string(const char* bytes, size_t len) {
    std::stringstream ss;
    ss << std::hex << std::setfill('0');
    for (size_t i = 0; i < len; ++i) {
        ss << std::setw(2) << static_cast<int>(static_cast<unsigned char>(bytes[i]));
    }
    return ss.str();
}

// Helper function to convert hex string to byte vector
std::vector<char> hex_string_to_bytes(const std::string& hex) {
    std::vector<char> bytes;
    if (hex.length() % 2 != 0) { // Basic validation: hex string must have even length
        std::cerr << "Warning: Hex string has odd length, ignoring last character." << std::endl;
    }
    for (size_t i = 0; i < hex.length() - (hex.length() % 2); i += 2) {
        std::string byteString = hex.substr(i, 2);
        try {
            char byte = static_cast<char>(std::stoul(byteString, nullptr, 16));
            bytes.push_back(byte);
        } catch (const std::invalid_argument& e) {
            std::cerr << "Warning: Invalid hex character sequence '" << byteString << "' skipped." << std::endl;
        } catch (const std::out_of_range& e) {
            std::cerr << "Warning: Hex character sequence '" << byteString << "' out of range, skipped." << std::endl;
        }
    }
    return bytes;
}


int main(int argc, char* argv[]) {
    if (argc < 2) {
        print_usage(argv[0]);
        return 1;
    }

    std::string operation = argv[1];

    if (operation == "compress") {
        std::string input_data;
        if (argc > 2) {
            input_data = argv[2];
        } else {
            // Check if stdin is coming from a pipe or redirect
            if (!isatty(fileno(stdin))) {
                 std::cerr << "Reading from stdin..." << std::endl;
                std::string line;
                while (std::getline(std::cin, line)) {
                    input_data += line + "\n";
                }
                // Remove the last newline if it was added by the loop and input wasn't empty
                if (!input_data.empty() && input_data.back() == '\n') {
                    input_data.pop_back();
                }
            } else {
                 // No argument and not a pipe, print usage
                std::cerr << "Error: Compress requires text input or data piped from stdin." << std::endl;
                print_usage(argv[0]);
                return 1;
            }
        }

        if (input_data.empty()) {
            std::cerr << "No input data provided for compression." << std::endl;
            return 1;
        }

        std::cout << "Original data length: " << input_data.length() << " bytes" << std::endl;
        // std::cout << "Original data: \"" << input_data << "\"" << std::endl; // Optional: print original data

        CompressedData compressed = compress_string(input_data.c_str(), input_data.length());
        if (compressed.buffer == nullptr) {
            std::cerr << "Compression failed! The returned buffer is null." << std::endl;
            // The Rust side might print more specific errors if compiled with verbose-errors
            return 1;
        }
         if (compressed.length == 0 && !input_data.empty()) {
            std::cerr << "Compression resulted in zero length, but input was not empty. This might indicate an error." << std::endl;
            // Potentially free and return, or let it proceed if 0-length is valid for some inputs
        }


        std::cout << "Compressed data length: " << compressed.length << " bytes" << std::endl;
        if (input_data.length() > 0) {
            std::cout << "Compression ratio: " << std::fixed << std::setprecision(2)
                      << (static_cast<double>(compressed.length) / input_data.length()) * 100.0
                      << "%" << std::endl;
        } else {
            std::cout << "Compression ratio: N/A (original data was empty)" << std::endl;
        }


        size_t preview_len = std::min(static_cast<size_t>(compressed.length), static_cast<size_t>(16));
        std::cout << "Compressed data (first " << preview_len << " bytes as hex): ";
        std::cout << bytes_to_hex_string(compressed.buffer, preview_len) << std::endl;

        std::string output_file = "compressed_output.bin";
        std::ofstream outfile(output_file, std::ios::binary);
        if (!outfile.is_open()) {
             std::cerr << "Error opening output file: " << output_file << std::endl;
             free_compressed_data(compressed);
             return 1;
        }
        outfile.write(compressed.buffer, compressed.length);
        outfile.close();
        std::cout << "Compressed data written to: " << output_file << std::endl;
        std::cout << "To decompress: " << argv[0] << " decompress " << output_file << std::endl;

        free_compressed_data(compressed);

    } else if (operation == "decompress") {
        if (argc < 3) {
            std::cerr << "Error: Decompress requires a file path." << std::endl;
            print_usage(argv[0]);
            return 1;
        }
        std::string file_path = argv[2];
        std::ifstream infile(file_path, std::ios::binary | std::ios::ate);
        if (!infile.is_open()) {
            std::cerr << "Error reading file '" << file_path << "'" << std::endl;
            return 1;
        }

        std::streamsize size = infile.tellg();
        infile.seekg(0, std::ios::beg);
        std::vector<char> buffer(size);
        if (!infile.read(buffer.data(), size)) {
            std::cerr << "Error reading file content from '" << file_path << "'" << std::endl;
            infile.close();
            return 1;
        }
        infile.close();

        if (buffer.empty()) {
            std::cerr << "Warning: Input file '" << file_path << "' is empty." << std::endl;
            // Decide if this is an error or should proceed. The Rust lib might handle it.
        }

        std::cout << "Compressed data length: " << buffer.size() << " bytes" << std::endl;

        DecompressedData decompressed = decompress_data(buffer.data(), buffer.size());
        if (decompressed.buffer == nullptr) {
            std::cerr << "Decompression failed! The returned buffer is null." << std::endl;
            // The Rust side might print more specific errors
            return 1;
        }

        std::cout << "Decompressed data length: " << decompressed.length << " bytes" << std::endl;
        std::cout << "Decompressed data: \"";
        // Ensure null termination for safety if printing directly, or print char by char
        std::string decompressed_str(decompressed.buffer, decompressed.length);
        std::cout << decompressed_str << "\"" << std::endl;
        
        std::string output_file = "decompressed_output.txt";
        std::ofstream outfile_text(output_file);
        if (!outfile_text.is_open()) {
             std::cerr << "Error opening output file: " << output_file << std::endl;
             free_decompressed_data(decompressed);
             return 1;
        }
        outfile_text.write(decompressed.buffer, decompressed.length);
        outfile_text.close();
        std::cout << "Decompressed data written to: " << output_file << std::endl;

        free_decompressed_data(decompressed);

    } else if (operation == "encode-varint") {
        if (argc < 3) {
            std::cerr << "Error: encode-varint requires a number." << std::endl;
            print_usage(argv[0]);
            return 1;
        }
        unsigned long number;
        try {
            number = std::stoul(argv[2]);
        } catch (const std::exception& e) {
            std::cerr << "Error: Invalid number format '" << argv[2] << "'. Please provide a valid unsigned 64-bit integer." << std::endl;
            return 1;
        }

        char varint_buffer[10]; // Max 10 bytes for u64 varint
        int32_t bytes_written = encode_varint(number, varint_buffer);

        if (bytes_written <= 0) { // Changed to <=0 as 0 could also be an error or unexpected
            std::cerr << "Error encoding varint. Code: " << bytes_written << std::endl;
            return 1;
        }
        std::cout << bytes_to_hex_string(varint_buffer, bytes_written) << std::endl;

    } else if (operation == "decode-varint") {
        if (argc < 3) {
            std::cerr << "Error: decode-varint requires hex bytes." << std::endl;
            print_usage(argv[0]);
            return 1;
        }
        std::string hex_str = argv[2];
        std::vector<char> bytes = hex_string_to_bytes(hex_str);

        if (bytes.empty() && !hex_str.empty() && hex_str != "00") { // "00" is a valid varint for 0
             std::cerr << "Error: Could not convert hex string '" << hex_str << "' to bytes, or hex string is invalid." << std::endl;
            return 1;
        }
         if (bytes.empty() && hex_str.empty()){
            std::cerr << "Error: Empty hex string provided for varint decoding." << std::endl;
            return 1;
        }


        unsigned long decoded_number;
        // Max bytes to read should be the size of our buffer, which is bytes.size()
        int32_t bytes_read = decode_varint(bytes.data(), static_cast<int32_t>(bytes.size()), &decoded_number);

        if (bytes_read <= 0) { // Changed to <=0 as 0 could also be an error or unexpected
            std::cerr << "Error decoding varint. Code: " << bytes_read << std::endl;
            if (bytes_read == -1) std::cerr << "(Possibly malformed VarInt or buffer too small)" << std::endl;
            if (bytes_read == -2) std::cerr << "(Possibly buffer too small to contain a full VarInt)" << std::endl;
            return 1;
        }
        std::cout << "Decoded number: " << decoded_number << std::endl;
        std::cout << "Bytes read: " << bytes_read << std::endl;

    } else {
        std::cerr << "Error: Unknown operation '" << operation << "'." << std::endl;
        print_usage(argv[0]);
        return 1;
    }

    return 0;
}
// Conditional includes for isatty/fileno are already attempted.
// If the direct includes above don't solve it, the issue might be more subtle.
// For now, the explicit includes for unistd.h and cstdio are the primary fix.
