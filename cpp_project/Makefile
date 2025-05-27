# Makefile for Rust FFI C++ Application

# Variables
RUST_PROJECT_DIR = ../rust_ffi_example
BUILD_DIR = build
CXX = g++
CXXFLAGS = -std=c++17 -Wall -Wextra -I$(RUST_PROJECT_DIR)
TARGET = cpp_app
SOURCE = main.cpp

# Determine build type
ifeq ($(BUILD_TYPE),debug)
	RUST_TARGET_DIR = $(RUST_PROJECT_DIR)/target/debug
	CARGO_FLAGS = 
	CXXFLAGS += -g -O0 -DDEBUG
else
	RUST_TARGET_DIR = $(RUST_PROJECT_DIR)/target/release
	CARGO_FLAGS = --release
	CXXFLAGS += -O3 -DNDEBUG
endif

# Rust library paths
RUST_LIB = $(RUST_TARGET_DIR)/lib$(shell basename $(RUST_PROJECT_DIR)).a

# Compression libraries (required by Rust FFI library)
COMPRESSION_LIBS = -lz -llz4 -lzstd

# Platform-specific libraries
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
	LDFLAGS = -ldl -lpthread $(COMPRESSION_LIBS)
endif
ifeq ($(UNAME_S),Darwin)
	LDFLAGS = -framework Security $(COMPRESSION_LIBS)
	# Add Homebrew paths for macOS
	CXXFLAGS += -I/opt/homebrew/include -I/usr/local/include
	LDFLAGS += -L/opt/homebrew/lib -L/usr/local/lib
endif

# Default target
.PHONY: all
all: $(TARGET)

# Build the Rust library
.PHONY: rust-lib
rust-lib:
	@echo "Building Rust library..."
	cd $(RUST_PROJECT_DIR) && cargo build $(CARGO_FLAGS)

# Build the C++ application
$(TARGET): rust-lib $(SOURCE)
	@echo "Building C++ application..."
	mkdir -p $(BUILD_DIR)
	$(CXX) $(CXXFLAGS) -o $(BUILD_DIR)/$(TARGET) $(SOURCE) $(RUST_LIB) $(LDFLAGS)
	@echo "Build completed: $(BUILD_DIR)/$(TARGET)"

# Clean targets
.PHONY: clean
clean:
	@echo "Cleaning build directory..."
	rm -rf $(BUILD_DIR)

.PHONY: clean-all
clean-all: clean
	@echo "Cleaning Rust target directory..."
	cd $(RUST_PROJECT_DIR) && cargo clean

# Test the application
.PHONY: test
test: $(TARGET)
	@echo "Testing the application..."
	./$(BUILD_DIR)/$(TARGET) compress "Hello, Rust FFI World!"

# Debug build
.PHONY: debug
debug:
	$(MAKE) BUILD_TYPE=debug

# Release build (default)
.PHONY: release
release:
	$(MAKE) BUILD_TYPE=release

# Install (copy to /usr/local/bin)
.PHONY: install
install: $(TARGET)
	@echo "Installing $(TARGET) to /usr/local/bin..."
	sudo cp $(BUILD_DIR)/$(TARGET) /usr/local/bin/

# Check dependencies
.PHONY: check-deps
check-deps:
	@echo "Checking dependencies..."
	@command -v cargo >/dev/null 2>&1 || { echo "Error: cargo is not installed"; exit 1; }
	@echo "Checking for compression libraries..."
	@pkg-config --exists zlib || echo "Warning: zlib not found via pkg-config"
	@pkg-config --exists liblz4 || echo "Warning: liblz4 not found via pkg-config"
	@pkg-config --exists libzstd || echo "Warning: libzstd not found via pkg-config"
	@echo "Dependencies check completed"

# Help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all        - Build the application (default)"
	@echo "  debug      - Build in debug mode"
	@echo "  release    - Build in release mode"
	@echo "  rust-lib   - Build only the Rust library"
	@echo "  test       - Build and test the application"
	@echo "  clean      - Clean C++ build directory"
	@echo "  clean-all  - Clean both C++ and Rust build directories"
	@echo "  check-deps - Check if all dependencies are available"
	@echo "  install    - Install the binary to /usr/local/bin"
	@echo "  help       - Show this help message" 