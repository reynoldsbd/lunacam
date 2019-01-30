#! /bin/bash
#
# Prepares root filesystem that will eventually be stored on the SD card
set -e
mnt=/mnt/img


# Just ignore the error code. A nonzero code from bsdtar does not indicate complete failure.
mkdir -p $mnt
bsdtar -xpf /arch-arm-base.tar.gz -C $mnt || true
