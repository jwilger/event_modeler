//! Build script for compiling libavoid and generating C++ bindings.

use std::path::Path;

fn main() {
    // TODO: Re-enable once libavoid submodule is initialized
    // Build libavoid from source
    // build_libavoid();

    // TODO: Generate C++ bindings once libclang is available
    // generate_bindings();
}

#[allow(dead_code)]
fn build_libavoid() {
    let libavoid_dir = Path::new("vendor/adaptagrams/cola/libavoid");

    if !libavoid_dir.exists() {
        panic!("libavoid source directory not found. Please ensure the submodule is initialized.");
    }

    // Collect all C++ source files
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(libavoid_dir).expect("Failed to read libavoid directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("cpp") {
            sources.push(path);
        }
    }

    // Build the C++ library
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++11")
        .include(libavoid_dir)
        .include("vendor/adaptagrams/cola") // For libavoid/header.h includes
        .define("LIBAVOID_STATIC", None)
        .flag_if_supported("-fPIC");

    // Add platform-specific flags
    if cfg!(target_os = "windows") {
        build.define("WIN32", None);
    }

    for source in sources {
        println!("cargo:rerun-if-changed={}", source.display());
        build.file(source);
    }

    build.compile("libavoid");

    // Tell cargo to link against our library
    println!("cargo:rustc-link-lib=static=libavoid");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // Re-run if any libavoid files change
    println!("cargo:rerun-if-changed=vendor/adaptagrams/cola/libavoid");
}

// TODO: Implement generate_bindings() once libclang is available
/*
fn generate_bindings() {
    let libavoid_dir = PathBuf::from("vendor/adaptagrams/cola/libavoid");

    let mut b = autocxx_build::Builder::new("src/routing/libavoid_ffi.rs", [&libavoid_dir])
        .build()
        .expect("Failed to build autocxx bindings");

    b.flag_if_supported("-std=c++11")
     .compile("autocxx-libavoid-bindings");

    println!("cargo:rerun-if-changed=src/routing/libavoid_ffi.rs");
}
*/
