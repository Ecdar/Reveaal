extern crate bindgen;

use tonic_build;

use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(feature = "dbm-stub") {
        println!("cargo:warning=Using stub instead of DBM library");
        return;
    }

    tonic_build::compile_protos("Ecdar-ProtoBuf/services.proto").unwrap();
    println!("cargo:rerun-if-changed=Ecdar-ProtoBuf/*.proto");

    let host = std::env::var("HOST").unwrap();
    let target = std::env::var("TARGET").unwrap();

    // Tell cargo to tell rustc to link the DBM
    // shared library.
    if host == target {
        println!("cargo:rustc-link-search=all=dbm/out/");
    } else {
        println!("cargo:rustc-link-search=all=dbm/out/{}/", target);
    }

    println!("cargo:rustc-link-lib=static=udbmwrapper");
    println!("cargo:rustc-link-lib=static=base");
    println!("cargo:rustc-link-lib=static=dbm");
    println!("cargo:rustc-link-lib=static=udebug");
    println!("cargo:rustc-link-lib=static=hash");
    println!("cargo:rustc-link-lib=stdc++");
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=dbm/wrapper.h");
    println!("cargo:rerun-if-changed=dbm/out");

    // cc::Build::new()
    //     .cpp(true)
    //     .file("dbm.cpp")
    //     .compile("dbm");
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("dbm/include/dbm/dbm.h")
        .header("dbm/include/dbm/fed.h")
        .header("dbm/wrapper.h")
        .trust_clang_mangling(true)
        .clang_args(&[
            "-x",
            "c++",
            "-std=c++14",
            "-fno-inline-functions",
            "-Idbm/include/",
        ])
        .allowlist_recursively(true)
        .generate_inline_functions(true)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        //avoid generating bindings for unused code that produces errors
        .opaque_type("namespace")
        .opaque_type("std::.*")
        //whitelist only relevant functions
        .allowlist_function("dbm_.*")
        .allowlist_function("fed_.*")
        // Enable comments for generated bindings
        .generate_comments(true)
        .detect_include_paths(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
