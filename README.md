# Reveaal

## About
This is a model checking engine for ECDAR (Environment for Compositional Design and Analysis of Real Time Systems) written in rust. 

## Prerequisites 
- A rust compiler and C++ compiler installed (https://www.rust-lang.org/learn/get-started) 
- [clang](https://clang.llvm.org/) for compiling the library (for linux, run the command ```apt install llvm-dev libclang-dev clang``` to properly install the clang dependencies)
- A folder containing the model components to check

## Building the DBM library

Download and compile the [UDBM v2.0.10](https://github.com/UPPAALModelChecker/UDBM/tree/cbb68a4a47c04f7e4e68fe78e16ba2069d894a28) library then copy all the archive files from ```build-Release/udbm/lib/```
into ```Reveaal/dbm/lib``` folder.
Navigate to the dbm folder and run the following command:
```./recompile.sh```
This compiles the library wrapper called libudbmwrapper.a.

## Building the project
- Build the project using `cargo build`
- Optionally run the tests using `cargo test -- --test-threads 1`


## Contact
Please contact any of the group members with any bugs/requests:

  abkr16@student.aau.dk
 
  tpede16@student.aau.dk
  
  eernst16@student.aau.dk
  
  pgreve16@student.aau.dk

