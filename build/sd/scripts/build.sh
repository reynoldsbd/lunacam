#! /bin/bash

# This script builds and customizes an Arch ARM image. It is designed to be run as root inside an
# Arch Linux docker image.

set -e


# Perform initialization within emulated Arch ARM system
/scripts/alarm-chroot.sh /img /scripts/alarm-init.sh
/scripts/alarm-chroot.sh /img /mnt/install.sh


####################################################################################################
# Convert filesystem contents into SD card image
#
# Although not the most efficient, this approach to packaging the skeleton image does not require
# special kernel support.
#
# Make sure to do some math before changing any of these block sizes.
####################################################################################################
img=/alarm.img

echo "build.sh: creating disk image skeleton"
rm -f $img
dd if=/dev/zero of=$img bs=2M count=1024 status=progress
sfdisk -q $img < /scripts/loop0.sfdisk

echo "build.sh: populating boot partition image"
mkfs.fat -F 32 -n BOOT -C /tmp/boot.img 102400 &> /dev/null
mcopy -i /tmp/boot.img -s /img/boot/* ::
dd if=/tmp/boot.img of=$img bs=512 seek=2048 conv=notrunc status=progress

echo "build.sh: populating root partition image"
rm -rf /img/boot/*
mkfs.ext4 -b 4096 -d /img -L root /tmp/root.img 498432
dd if=/tmp/root.img of=$img bs=512 seek=206848 conv=notrunc status=progress

echo "build.sh: syncing disks"
sync

echo "build.sh: $img is ready"
