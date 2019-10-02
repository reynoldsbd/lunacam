#!/bin/bash

set -e

export CROSSBUILD=1
export RUST_TARGET=arm-unknown-linux-gnueabihf
export RUST_PROFILE=release

make $1

chown -R $OUT_UID:$OUT_GID ./build/
