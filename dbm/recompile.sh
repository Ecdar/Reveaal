#!/bin/sh
set -xe

export CXX=${CXX:-g++}

mkdir objectFiles || true
rm objectFiles/*.o libudbmwrapper.a || true

if [ ! -e libudbm.a ]; then
  echo "Missing libudbm.a"
  exit 1
fi

# Extract object files from libudbm.a
(
  cd objectFiles
  ar x ../libudbm.a
)

# Compile wrapper
$CXX -c wrapper.cpp -I include/ -o objectFiles/wrapper.o

# Join wrapper and libudbm.a into a library archive
ar rvs libudbmwrapper.a objectFiles/*.o
