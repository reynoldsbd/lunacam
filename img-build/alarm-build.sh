#! /bin/bash
#
# This script builds and customizes an Arch ARM image. It is designed to be run as root inside an
# Arch Linux docker image.
set -e


####################################################################################################
# Perform image customization inside ARM system
####################################################################################################

echo "alarm-build > Customizing ARM system"
/scripts/alarm-chroot.sh /img /scripts/alarm-init.sh


####################################################################################################
# Convert filesystem contents into SD card image
#
# Although not the most efficient, this approach to packaging the skeleton image does not require
# special kernel support.
#
# Make sure to do some math before changing any of these block sizes.
####################################################################################################
img=/alarm.img

echo "alarm-build > Creating disk image"
rm -f $img
dd if=/dev/zero of=$img bs=2M count=1024 &> /dev/null
sfdisk $img < /scripts/loop0.sfdisk &> /dev/null

echo "alarm-build > Building boot partition image"
mkfs.fat -F 32 -n BOOT -C /tmp/boot.img 102400 &> /dev/null
mcopy -i /tmp/boot.img -s /img/boot/* ::
rm -rf /img/boot/*
dd if=/tmp/boot.img of=$img bs=512 seek=2048 conv=notrunc &> /dev/null

echo "alarm-build > Building root partition image"
mkfs.ext4 -b 4096 -d /img -L root /tmp/root.img 498432
echo "FOO"
dd if=/tmp/root.img of=$img bs=512 seek=206848 conv=notrunc

echo "alarm-build > Flushing disk cache"
sync

echo "alarm-build > Disk image is ready!"
