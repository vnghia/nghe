set(VCPKG_TARGET_ARCHITECTURE "arm64")
set(VCPKG_CRT_LINKAGE "static")
set(VCPKG_LIBRARY_LINKAGE "static")

set(VCPKG_CMAKE_SYSTEM_NAME "Linux")

set(CMAKE_CXX_COMPILER "aarch64-alpine-linux-musl-c++")
set(CMAKE_C_COMPILER "aarch64-alpine-linux-musl-cc")
set(CMAKE_HOST_SYSTEM_PROCESSOR "aarch64")

set(VCPKG_C_FLAGS "-static -static-libgcc")
set(VCPKG_CXX_FLAGS "-static -static-libgcc")
set(VCPKG_LINKER_FLAGS "-fuse-ld=mold")
