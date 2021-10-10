# Reveaal

## About
This is a model checking engine for ECDAR (Environment for Compositional Design and Analysis of Real Time Systems) written in rust. 

## Prerequisites 
- A rust compiler and C++ compiler installed (https://www.rust-lang.org/learn/get-started) 
- [clang](https://clang.llvm.org/) for compiling the library (for linux, run the command ```apt install llvm-dev libclang-dev clang protobuf-compiler``` to properly install the clang dependencies)
- A folder containing the model components to check
- ProtoBuf compiler protoc (linux ```apt install protobuf-compiler```)

## Building the DBM library

Navigate into the dbm folder and make sure no build folder already exists. Then use cmake to build the dbm library, the files should be created in a `out` folder where Reveaal use them from.

`cmake -B build/`

`cmake --build build/`

## Building the project
- Build the project using `cargo build`
- Optionally run the tests using `cargo test -- --test-threads 1`

## Cross compiling from Linux to Windows
Ensure the build directory doesnt already exist, before starting.
Build the dbm library with using the x86_64-w64-mingw32 toolchain.

`cmake -B build/ -D CMAKE_TOOLCHAIN_FILE=toolchain-x86_64-w64-mingw32.cmake`

`cmake --build build/`

`cargo build --target x86_64-pc-windows-gnu`

The rust compiler is configured to pick the library for the current target, so you dont have to recompile the dbm library when switching compile target.

## Contact
Please contact any of the group members with any bugs/requests:

  abkr16@student.aau.dk
 
  tpede16@student.aau.dk
  
  eernst16@student.aau.dk
  
  pgreve16@student.aau.dk

