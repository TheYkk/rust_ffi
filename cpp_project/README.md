# Rust FFI C++ Application

This C++ application demonstrates how to use a Rust library through FFI (Foreign Function Interface). The application provides compression/decompression functionality and variable-length integer encoding/decoding using algorithms implemented in Rust.

## Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **CMake**: Version 3.15 or higher
- **C++ Compiler**: GCC, Clang, or MSVC with C++17 support
- **Make**: For using the Makefile (optional)
- **Compression Libraries**: zlib, lz4, and zstd development packages

### Installing Dependencies

#### macOS (using Homebrew)
```bash
brew install cmake lz4 zstd
# zlib is included with macOS
```

#### Ubuntu/Debian
```bash
sudo apt-get install cmake build-essential zlib1g-dev liblz4-dev libzstd-dev pkg-config
```

#### Fedora/CentOS
```bash
sudo dnf install cmake gcc-c++ zlib-devel lz4-devel libzstd-devel pkgconfig
```

## Project Structure

```
cpp_project/
├── main.cpp              # C++ application source
├── CMakeLists.txt        # CMake configuration
├── Makefile              # Alternative build system
├── build.sh              # Automated build script
└── README.md             # This file

../rust_ffi_example/      # Rust library (FFI interface)
├── src/                  # Rust source code
├── Cargo.toml            # Rust project configuration
└── rust_ffi_example.h    # C++ header for FFI functions
```

## Building the Application

### Option 1: Using the Build Script (Recommended)

The easiest way to build the application:

```bash
# Build in release mode (default)
./build.sh

# Build in debug mode
./build.sh --debug

# Clean build and rebuild
./build.sh --clean

# Verbose output
./build.sh --verbose

# Show help
./build.sh --help
```

### Option 2: Using CMake

```bash
# Create build directory
mkdir -p build
cd build

# Configure and build (Release)
cmake ..
cmake --build .

# Or configure for Debug
cmake -DCMAKE_BUILD_TYPE=Debug ..
cmake --build .
```

### Option 3: Using Make

```bash
# Build in release mode (default)
make

# Build in debug mode
make debug

# Clean and rebuild
make clean-all && make

# Test the application
make test

# Check dependencies
make check-deps

# Show available targets
make help
```

## Usage

The built executable `cpp_app` provides several commands:

### Compression/Decompression

```bash
# Compress text
./build/cpp_app compress "Hello, World!"

# Compress from stdin
echo "Hello from pipe" | ./build/cpp_app compress

# Decompress a file
./build/cpp_app decompress compressed_output.bin
```

### Variable-Length Integer Encoding

```bash
# Encode a number to varint format
./build/cpp_app encode-varint 12345

# Decode varint hex bytes to number
./build/cpp_app decode-varint b960
```

## How It Works

1. **Rust Library**: The `rust_ffi_example` crate provides compression and encoding functions with a C-compatible interface
2. **FFI Bridge**: The `rust_ffi_example.h` header defines the C++ interface to Rust functions
3. **CMake Integration**: CMake automatically builds the Rust library before compiling the C++ application
4. **Static Linking**: The Rust library is compiled as a static library and linked into the C++ executable
5. **Dependency Management**: The build system automatically finds and links required compression libraries (zlib, lz4, zstd)

## Build Process Details

The build system:

1. Uses pkg-config to find compression libraries (zlib, lz4, zstd)
2. Builds the Rust library using `cargo build`
3. Links the resulting static library (`librust_ffi_example.a`) to the C++ application
4. Includes platform-specific system libraries (Security framework on macOS, dl/pthread on Linux)
5. Ensures proper dependency management between Rust and C++ components

## Troubleshooting

### Common Issues

**Rust not found**: Ensure Rust is installed and `cargo` is in your PATH
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**CMake not found**: Install CMake for your platform
```bash
# macOS
brew install cmake

# Ubuntu/Debian
sudo apt-get install cmake

# Windows
# Download from https://cmake.org/download/
```

**Compression libraries not found**: Install the development packages
```bash
# macOS
brew install lz4 zstd

# Ubuntu/Debian
sudo apt-get install liblz4-dev libzstd-dev zlib1g-dev

# Fedora/CentOS
sudo dnf install lz4-devel libzstd-devel zlib-devel
```

**Build fails on macOS**: Ensure Xcode command line tools are installed
```bash
xcode-select --install
```

**Linking errors**: Try cleaning all build artifacts and rebuilding
```bash
./build.sh --clean
```

**pkg-config not found**: Install pkg-config
```bash
# macOS
brew install pkg-config

# Ubuntu/Debian
sudo apt-get install pkg-config

# Fedora/CentOS
sudo dnf install pkgconfig
```

## Development

### Modifying the Rust Library

1. Make changes in `../rust_ffi_example/src/`
2. The build system will automatically rebuild the Rust library
3. Rebuild the C++ application

### Adding New FFI Functions

1. Add the function to `../rust_ffi_example/src/lib.rs`
2. Update `../rust_ffi_example/rust_ffi_example.h`
3. Use the function in `main.cpp`
4. Rebuild the project

## Performance Notes

- Release builds are significantly faster than debug builds
- The Rust library is optimized with LTO (Link Time Optimization) in release mode
- Static linking eliminates runtime dependencies but increases executable size

## Example Output

```bash
$ ./build/cpp_app compress "Hello, Rust FFI World!"
Original data length: 22 bytes
Compressed data length: 31 bytes
Compression ratio: 140.91%
Compressed data (first 16 bytes as hex): 16789cf348cdc9c9d751082a2d2e5170
Compressed data written to: compressed_output.bin
To decompress: ./build/cpp_app decompress compressed_output.bin

$ ./build/cpp_app encode-varint 12345
b960

$ ./build/cpp_app decode-varint b960
Decoded number: 12345
Bytes read: 2
```

## License

This project demonstrates FFI integration techniques and is provided as an example. 