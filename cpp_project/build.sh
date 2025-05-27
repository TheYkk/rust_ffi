#!/bin/bash

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    print_error "cargo is not installed. Please install Rust and cargo."
    exit 1
fi

# Check if cmake is available
if ! command -v cmake &> /dev/null; then
    print_error "cmake is not installed. Please install CMake."
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Parse command line arguments
BUILD_TYPE="Release"
CLEAN=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_TYPE="Debug"
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --debug     Build in debug mode (default: release)"
            echo "  --clean     Clean build directories before building"
            echo "  --verbose   Enable verbose output"
            echo "  -h, --help  Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_status "Building Rust FFI C++ Application"
print_status "Build type: $BUILD_TYPE"

# Clean if requested
if [ "$CLEAN" = true ]; then
    print_status "Cleaning build directories..."
    
    # Clean Rust build
    if [ -d "../rust_ffi_example" ]; then
        cd "../rust_ffi_example"
        cargo clean
        cd "$SCRIPT_DIR"
        print_status "Cleaned Rust build directory"
    fi
    
    # Clean CMake build
    if [ -d "build" ]; then
        rm -rf build
        print_status "Cleaned CMake build directory"
    fi
fi

# Create build directory
mkdir -p build
cd build

# Configure CMake
print_status "Configuring CMake..."
CMAKE_ARGS="-DCMAKE_BUILD_TYPE=$BUILD_TYPE"
if [ "$VERBOSE" = true ]; then
    CMAKE_ARGS="$CMAKE_ARGS -DCMAKE_VERBOSE_MAKEFILE=ON"
fi

cmake .. $CMAKE_ARGS

# Build the project
print_status "Building project..."
if [ "$VERBOSE" = true ]; then
    cmake --build . --config $BUILD_TYPE --verbose
else
    cmake --build . --config $BUILD_TYPE
fi

print_status "Build completed successfully!"
print_status "Executable location: $(pwd)/cpp_app"

# Test if the executable was created
if [ -f "cpp_app" ]; then
    print_status "Testing the executable..."
    echo
    echo "=== Testing compression functionality ==="
    ./cpp_app compress "Hello, Rust FFI World!"
    echo
    print_status "Build and basic test completed successfully!"
else
    print_error "Executable 'cpp_app' was not created!"
    exit 1
fi 