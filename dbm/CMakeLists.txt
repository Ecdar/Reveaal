CMAKE_MINIMUM_REQUIRED(VERSION 3.16.3)
include(ExternalProject)

project(dbm_wrapper)


set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/out/${triple})
message(Compiling to output directory ${CMAKE_ARCHIVE_OUTPUT_DIRECTORY})

set(dbm_patch ${CMAKE_SOURCE_DIR}/DBMAlloc.patch)


ExternalProject_Add(udbm PREFIX UDBM/
    GIT_REPOSITORY https://github.com/UPPAALModelChecker/UDBM
    GIT_TAG v2.0.10
    GIT_PROGRESS true
    PATCH_COMMAND git apply ${dbm_patch}
    INSTALL_COMMAND ${CMAKE_COMMAND} -E copy_directory udbm/lib ${CMAKE_ARCHIVE_OUTPUT_DIRECTORY}
    CMAKE_ARGS -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE}
)

include_directories(include/)
add_library(udbmwrapper STATIC wrapper.cpp)
add_dependencies(udbmwrapper udbm)