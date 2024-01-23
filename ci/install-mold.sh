#!/usr/bin/env bash

MOLD_VERSION=2.4.0
curl -L --retry 10 https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-x86_64-linux.tar.gz | tar -C /usr/ --strip-components=1 --no-overwrite-dir -xzf -
/usr/bin/mold --version
