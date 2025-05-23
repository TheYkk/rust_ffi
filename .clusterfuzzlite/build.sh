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

# Note: This script is running in $SRC/rust_ffi_example
# Find all fuzz targets in the fuzz/fuzz_targets directory by reading the Cargo.toml file.
FUZZ_TARGETS=$(cd fuzz && cargo read-manifest | jq -r '.targets[] | select(.kind[] | contains("bin")) | select(.name | startswith("fuzz_")) | .name')

# Build all fuzz targets.
# The `cargo fuzz build` command does not support building multiple targets when specifying sanitizers.
# So we need to loop through the fuzz targets and build them one by one.
for target in $FUZZ_TARGETS
do
  cargo fuzz build -O \
      -s address,undefined \
      $target
done

# Copy the fuzzer executables to $OUT.
for target in $FUZZ_TARGETS
do
  cp fuzz/target/x86_64-unknown-linux-gnu/release/$target $OUT/$target
done
