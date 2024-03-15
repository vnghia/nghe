set -euxo pipefail

ARCH=$(uname -m)

if [ "$ARCH" = "x86_64" ]; then
  export VCPKG_DEFAULT_HOST_TRIPLET=x64-linux
else
  export VCPKG_DEFAULT_HOST_TRIPLET=arm64-linux-musl
fi

cargo vcpkg --verbose build --target ${ARCH}-unknown-linux-musl
