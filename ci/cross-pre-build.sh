#!/usr/bin/env bash
set -euxo pipefail

if [[ "$CROSS_TARGET" == "aarch64-unknown-linux-gnu" ]]; then
  dpkg --add-architecture $CROSS_DEB_ARCH
  apt-get update && apt-get --assume-yes install libpq-dev:$CROSS_DEB_ARCH
fi

MOLD_VERSION=2.4.0
curl -L --retry 10 https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-x86_64-linux.tar.gz | tar -C /usr/ --strip-components=1 --no-overwrite-dir -xzf -
