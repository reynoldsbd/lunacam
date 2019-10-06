#!/bin/bash

set -e

export CROSSBUILD=1
export LC_TARGET=arm-unknown-linux-gnueabihf
export LC_PROFILE=release

make $1

chown -R $OUT_UID:$OUT_GID ./build/
