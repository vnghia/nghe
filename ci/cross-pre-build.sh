#!/usr/bin/env bash
set -euxo pipefail

if [[ "$CROSS_TARGET" == "aarch64-unknown-linux-gnu" ]]; then
  apt-get update && apt-get --assume-yes install libclang-10-dev
fi
