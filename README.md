# Reveaal

## About
This is a model checking engine for ECDAR (Environment for Compositional Design and Analysis of Real Time Systems) written in rust. 

## Prerequisites 
- A rust compiler and C++ compiler installed (https://www.rust-lang.org/learn/get-started) 
- [clang](https://clang.llvm.org/) for compiling the library (for the Ubuntu linux distribution, run the command ```apt install llvm-dev libclang-dev clang``` to properly install the clang dependencies)
- A folder containing the model components to check
- ProtoBuf compiler protoc (for the Ubuntu linux distribution ```apt install protobuf-compiler```)
- Download ProtoBuf definitions with ```git submodule update --init --recursive```

## Building the DBM library

Navigate into the dbm folder and make sure no build folder already exists. Then use cmake to build the dbm library, the files should be created in a `out` folder where Reveaal use them from.

```bash
cd dbm
cmake -B build/
cmake --build build/
```

## Building the project
- Build the project using `cargo build`
- Optionally run the tests using `cargo test`

## Cross compiling from Linux to Windows
Build the DBM library using the x86_64-w64-mingw32 toolchain:

```bash
cd dbm
cmake -B buildw/ -D CMAKE_TOOLCHAIN_FILE=toolchain-x86_64-w64-mingw32.cmake
cmake --build buildw/
```

Ensure that the rustc windows target is installed with `rustup target add x86_64-pc-windows-gnu` and build with cargo:

`cargo build --target x86_64-pc-windows-gnu`

The rust compiler is configured to pick the library for the current target, so you dont have to recompile the dbm library when switching compile target.
