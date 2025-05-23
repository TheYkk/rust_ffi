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

FROM gcr.io/oss-fuzz-base/base-builder-rust:v1

# Install zlib1g-dev for the zlib.h header file.
RUN apt-get update && apt-get install -y zlib1g-dev

# Install nightly toolchain and set it as default.
# The base-builder-rust image should have rustup pre-installed.
RUN rustup toolchain install nightly &&     rustup default nightly &&\
    # Ensure cargo-fuzz is installed for the nightly toolchain
    cargo +nightly install cargo-fuzz &&\
    # Install components that might be needed by fuzzing or specific crates
    rustup component add rust-src --toolchain nightly &&\
    rustup component add llvm-tools-preview --toolchain nightly &&\
    rustup component add clippy --toolchain nightly

# Copy the project's source code.
# Assuming the context of the Docker build is the root of the repository.
COPY . $SRC/
# Copy the build script.
COPY ./.clusterfuzzlite/build.sh $SRC/
# Workdir for build script.
WORKDIR $SRC/rust_ffi_example
