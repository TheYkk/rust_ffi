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
            // Zlib linking is handled by pkg-config if found, or attempted directly below.
        }
        Err(_) => {
            // Fallback if pkg-config fails or zlib is not found by it.
            println!("cargo:warning=pkg-config failed to find zlib, attempting to link with -lz directly. This might fail if zlib is not in the default search paths.");
            // We still need to compile our C code.
            println!("cargo:rerun-if-changed=src/clib.c");
            let mut build = cc::Build::new();
            build.file("src/clib.c");
            
            // Enable verbose error messages if the feature is enabled
            if verbose_errors {
                build.define("DEBUG_FUZZING", "1");
                println!("cargo:warning=Building with verbose error messages enabled (zlib pkg-config failed path)");
            }
            
            build.compile("clib"); // Output will be libclib.a
            println!("cargo:rustc-link-lib=z"); // Link against zlib
        }
    }

    // Check if liblz4 is available on the system
    match pkg_config::probe_library("liblz4") {
        Ok(_) => {
            // If liblz4 is found by pkg-config, it will handle the linking.
            // No specific rustc-link-lib needed here as pkg-config does it.
            println!("cargo:rerun-if-changed=src/clib.c"); // Ensure C code is recompiled if it changes
        }
        Err(_) => {
            // Fallback if pkg-config fails or liblz4 is not found by it.
            println!("cargo:warning=pkg-config failed to find liblz4, attempting to link with -llz4 directly. This might fail if liblz4 is not in the default search paths.");
            println!("cargo:rustc-link-lib=lz4"); // Link against liblz4
            println!("cargo:rerun-if-changed=src/clib.c"); // Ensure C code is recompiled if it changes
        }
    }

    // The C code compilation is handled in the zlib section.
    // If zlib pkg-config succeeded, clib.c is compiled there.
    // If zlib pkg-config failed, clib.c is also compiled in that fallback block.
    // We only need to ensure that if for some reason a separate compilation pass was needed,
    // it would also include the DEBUG_FUZZING define.
    // However, cc::Build is configured and run only once based on zlib's pkg-config status.
    // The rerun-if-changed for clib.c is important and is present in both zlib and lz4 blocks.
}
