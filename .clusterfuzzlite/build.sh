#!/bin/bash -eu
# Copyright 2023 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
################################################################################

# This script is executed from $SRC/rust_ffi_example (due to WORKDIR in Dockerfile)

# Find all fuzz targets in the fuzz/fuzz_targets directory by reading the Cargo.toml file.
# The 'cd fuzz' is relative to the WORKDIR $SRC/rust_ffi_example
FUZZ_TARGETS=$(cd fuzz && cargo +nightly read-manifest | jq -r '.targets[] | select(.kind[] | contains("bin")) | select(.name | startswith("fuzz_")) | .name')

# Unset RUSTFLAGS before calling cargo fuzz build.
# cargo-fuzz sets its own RUSTFLAGS, and conflicting flags from the environment
# (e.g., from the ClusterFuzzLite base image) can cause issues.
unset RUSTFLAGS

cargo version
rustc --version
rustup --version
rustup show
cargo fuzz --version
echo "==="
cargo +nightly version
rustc +nightly --version
rustup +nightly --version
rustup +nightly show
cargo +nightly fuzz --version
# Build all fuzz targets for each sanitizer using the nightly toolchain.
# The SANITIZER environment variable is set by ClusterFuzzLite.
# cargo fuzz build only supports one sanitizer at a time.
# Disable LTO for release profile to avoid linking issues with sanitizers.
for target in $FUZZ_TARGETS
do
  echo "Building fuzz target: $target with sanitizer: $SANITIZER using nightly Rust and LTO disabled"
  # Ensure context for cargo fuzz build is the 'fuzz' directory where its Cargo.toml is located.
  (cd fuzz && CARGO_PROFILE_RELEASE_LTO=false cargo +nightly fuzz build -O \
      -s $SANITIZER \
      $target)
done

# Copy the fuzzer executables to $OUT.
# The executables are created in $SRC/rust_ffi_example/fuzz/target/x86_64-unknown-linux-gnu/release/
for target in $FUZZ_TARGETS
do
  cp "fuzz/target/x86_64-unknown-linux-gnu/release/$target" "$OUT/$target"
done
