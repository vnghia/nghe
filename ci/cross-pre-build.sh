#!/usr/bin/env bash
set -euxo pipefail

if [[ "$CROSS_TARGET" == "aarch64-unknown-linux-gnu" ]]; then
  dpkg --add-architecture $CROSS_DEB_ARCH
  apt-get update && apt-get --assume-yes install libpq-dev:$CROSS_DEB_ARCH \
    libclang-10-dev
fi
