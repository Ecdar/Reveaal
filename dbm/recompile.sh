#!/bin/sh
set -xe

#Check for missing files
if [ ! -e lib/libbase.a ]; then
  echo "Missing lib/libbase.a"
  exit 1
fi

if [ ! -e lib/libdbm.a ]; then
  echo "Missing lib/libdbm.a"
  exit 1
fi

if [ ! -e lib/libhash.a ]; then
  echo "Missing lib/libhash.a"
  exit 1
fi

if [ ! -e lib/libudebug.a ]; then
  echo "Missing lib/libudebug.a"
  exit 1
fi

#Clean
mkdir objectFiles || true
rm objectFiles/*.o libudbmwrapper.a || true

# Extract object files from lib folder
(
  cd objectFiles
  ar x ../lib/libbase.a
  ar x ../lib/libdbm.a
  ar x ../lib/libhash.a
  ar x ../lib/libudebug.a
)

# Compile wrapper
g++ -c wrapper.cpp -I include/ -o objectFiles/wrapper.o

# Join wrapper and libudbm.a into a library archive
ar rvs libudbmwrapper.a objectFiles/*.o