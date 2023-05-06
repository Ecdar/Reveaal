# Reveaal

## About
This is a model checking engine for ECDAR (Environment for Compositional Design and Analysis of Real Time Systems) written in rust. 

#### DBM Library
The engine uses the ECDAR DBM Library for operations on zones of time (https://www.github.com/ECDAR/EDBM).

## Prerequisites 
- A rust compiler installed (https://www.rust-lang.org/learn/get-started)
- A folder containing the model components to check
- ProtoBuf compiler protoc (for the Ubuntu linux distribution ```apt install protobuf-compiler```)
- Download ProtoBuf definitions with ```git submodule update --init --recursive```

## Building the project
- Build the project using `cargo build`
- Optionally run the tests using `cargo test`

## Cross compiling
The project is pure Rust so one should be able to crosscompile to any platform with a rust target.

### Compiling to Windows
Make sure you have mingw installed `sudo apt-get install mingw-w64` and the rustc windows target is installed with `rustup target add x86_64-pc-windows-gnu` and build with cargo:
`cargo build --target x86_64-pc-windows-gnu`