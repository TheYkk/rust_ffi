extern crate cc;

fn main() {
    // Check if verbose-errors feature is enabled
    let verbose_errors = cfg!(feature = "verbose-errors");
    
    // Check if zlib is available on the system
    match pkg_config::probe_library("zlib") {
        Ok(_) => {
            // If zlib is found by pkg-config, it will handle the linking.
            // We still need to compile our C code.
            println!("cargo:rerun-if-changed=src/clib.c");
            let mut build = cc::Build::new();
            build.file("src/clib.c");
            
            // Enable verbose error messages if the feature is enabled
            if verbose_errors {
                build.define("DEBUG_FUZZING", "1");
                println!("cargo:warning=Building with verbose error messages enabled");
            }
            
            build.compile("clib");
            println!("cargo:rustc-link-lib=z"); // Link against zlib
        }
        Err(_) => {
            // Fallback if pkg-config fails or zlib is not found by it.
            // This assumes zlib headers are in a standard location and zlib is linkable as `libz`.
            // For more robust builds, especially cross-platform, consider using vcpkg or other methods.
            println!("cargo:warning=pkg-config failed to find zlib, attempting to link with -lz directly. This might fail if zlib is not in the default search paths.");
            println!("cargo:rerun-if-changed=src/clib.c");
            let mut build = cc::Build::new();
            build.file("src/clib.c");
            
            // Enable verbose error messages if the feature is enabled
            if verbose_errors {
                build.define("DEBUG_FUZZING", "1");
                println!("cargo:warning=Building with verbose error messages enabled");
            }
            
            build.compile("clib"); // Output will be libclib.a
            println!("cargo:rustc-link-lib=z"); // Link against zlib
        }
    }
}
