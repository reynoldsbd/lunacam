#!/bin/bash

# This script builds a specified variant (daemon or portal) of the LunaCam SD card image. It is
# designed to be run inside the imagebuild Docker image.

set -e
variant=$1
image=/source/build/images/lc-$variant.img

echo "build.sh: starting image setup"
/source/tools/imagebuild/alarm-chroot.sh /img /source/tools/imagebuild/setup.sh $variant

echo "build.sh: creating image skeleton"
rm -f $image
mkdir -p $(dirname $image)
dd if=/dev/zero of=$image bs=2M count=1024 status=progress
sfdisk -q $image < /source/tools/imagebuild/loop0.sfdisk

echo "build.sh: populating boot partition"
mkfs.fat -F 32 -n BOOT -C /tmp/boot.img 102400 &> /dev/null
mcopy -i /tmp/boot.img -s /img/boot/* ::
dd if=/tmp/boot.img of=$image bs=512 seek=2048 conv=notrunc status=progress

echo "build.sh: populating root partition"
rm -rf /img/boot/*
mkfs.ext4 -b 4096 -d /img -L root /tmp/root.img 498432
dd if=/tmp/root.img of=$image bs=512 seek=206848 conv=notrunc status=progress

echo "build.sh: flushing changes to disk"
sync

echo "build.sh: $image is ready"
