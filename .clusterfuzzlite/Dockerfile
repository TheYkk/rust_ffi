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

# Install rustup and set nightly as default.
# The base-builder-rust image might have a specific version of Rust.
# We install rustup to manage toolchains and switch to nightly.
ENV RUSTUP_HOME=/usr/local/rustup     CARGO_HOME=/usr/local/cargo     PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
RUN rustup install nightly &&     rustup default nightly &&     # Install components that might be needed by fuzzing or specific crates
    rustup component add rust-src llvm-tools-preview &&     # Ensure cargo-fuzz is installed for the nightly toolchain
    cargo install cargo-fuzz

# Copy the project's source code.
COPY . $SRC/
# Copy the build script.
COPY ./.clusterfuzzlite/build.sh $SRC/
# Workdir for build script.
WORKDIR $SRC/rust_ffi_example
