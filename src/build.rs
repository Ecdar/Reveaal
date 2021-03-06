extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(feature = "dbm-stub") {
        println!("cargo:warning=Using stub instead of DBM library");
        return;
    }

    // Tell cargo to tell rustc to link the DBM
    // shared library.
    println!("cargo:rustc-link-search=native=dbm/");
    println!("cargo:rustc-link-lib=udbmwrapper");
    println!("cargo:rustc-link-lib=stdc++");
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=dbm/wrapper.h");

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
        .header("dbm/include/dbm/constraints.h")
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
        .allowlist_function("constraint_t")
        .allowlist_function("constrain")
        .allowlist_function("subtractDown")
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
