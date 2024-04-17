#!/usr/bin/env bash
set -euxo pipefail

if [[ "$CROSS_TARGET" == "aarch64-unknown-linux-gnu" ]]; then
  dpkg --add-architecture $CROSS_DEB_ARCH
  apt-get update && apt-get --assume-yes install libpq-dev:$CROSS_DEB_ARCH \
    libclang-10-dev
elif [[ "$CROSS_TARGET" == "x86_64-unknown-freebsd" ]]; then
  # Issue: https://github.com/cross-rs/cross/issues/1367
  mkdir -p /usr/local/x86_64-unknown-freebsd13/usr &&
    ln -s ../include /usr/local/x86_64-unknown-freebsd13/usr/include &&
    ln -s ../lib /usr/local/x86_64-unknown-freebsd13/usr/lib
fi
