extern crate cc;
extern crate pkg_config;

use std::path::Path;

/// Attempts to find a library using pkg-config. If successful, it adds the
/// include paths to the cc::Build instance. If pkg-config fails, it attempts
/// a fallback mechanism: linking manually and searching common include paths.
///
/// Returns `true` if the library and its headers are found (either via
/// pkg-config or fallback), `false` otherwise.
fn find_and_add_library(build: &mut cc::Build, pkg_name: &str, lib_name: &str, header_name: &str) -> bool {
    // Try finding the library using pkg-config
    match pkg_config::probe_library(pkg_name) {
        Ok(lib) => {
            // Add include paths found by pkg-config to the C compiler build
            for path in lib.include_paths {
                build.include(path);
            }
            // pkg_config automatically tells cargo how to link, so we're done.
            true
        }
        Err(_) => {
            println!("cargo:warning=pkg-config failed to find {}, attempting fallback...", pkg_name);
            // Fallback: Manually tell cargo to link the library.
            println!("cargo:rustc-link-lib={}", lib_name);

            // Fallback: Search common include paths for the header.
            let common_paths = [
                "/usr/include",
                "/usr/local/include",
                "/opt/homebrew/include", // For Homebrew on Apple Silicon/Intel
                "/opt/local/include",    // For MacPorts
            ];

            for path_str in &common_paths {
                let path = Path::new(path_str);
                let header = path.join(header_name);
                if header.exists() {
                    build.include(path);
                    println!("cargo:warning=Found {} headers at {}. Fallback successful.", pkg_name, path.display());
                    return true; // Found via fallback
                }
            }

            println!("cargo:warning=Fallback failed to find {} headers.", pkg_name);
            false // Library not found
        }
    }
}

fn main() {
    let mut build = cc::Build::new();
    build.file("src/clib.c"); // Specify the C source file

    // Handle the 'verbose-errors' feature flag
    if cfg!(feature = "verbose-errors") {
        build.define("DEBUG_FUZZING", "1");
        println!("cargo:warning=Building with verbose error messages enabled (DEBUG_FUZZING=1).");
    }

    // Find and configure zlib
    if !find_and_add_library(&mut build, "zlib", "z", "zlib.h") {
        // Panic with instructions if zlib isn't found
        panic!("zlib library or headers not found. \
                Please install the zlib development package (e.g., 'zlib1g-dev' on Debian/Ubuntu, \
                'zlib-devel' on Fedora/CentOS, or 'zlib' via Homebrew/MacPorts) \
                or ensure pkg-config can locate it.");
    }

    // Find and configure lz4
    if !find_and_add_library(&mut build, "liblz4", "lz4", "lz4.h") {
        // Panic with instructions if lz4 isn't found
        panic!("lz4 library or headers not found. \
                Please install the lz4 development package (e.g., 'liblz4-dev' on Debian/Ubuntu, \
                'lz4-devel' on Fedora/CentOS, or 'lz4' via Homebrew/MacPorts) \
                or ensure pkg-config can locate it.");
    }

    // Find and configure zstd
    if !find_and_add_library(&mut build, "libzstd", "zstd", "zstd.h") {
        // Panic with instructions if zstd isn't found
        panic!("zstd library or headers not found. \
                Please install the zstd development package (e.g., 'libzstd-dev' on Debian/Ubuntu, \
                'zstd-devel' on Fedora/CentOS, or 'zstd' via Homebrew/MacPorts) \
                or ensure pkg-config can locate it.");
    }

    // Ensure Cargo reruns this script if the C file changes
    println!("cargo:rerun-if-changed=src/clib.c");

    // Compile the C library
    build.compile("clib");

}