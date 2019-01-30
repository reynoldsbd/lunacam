#! /bin/bash
#
# This script builds and customizes an Arch ARM image. It is designed to be run as root inside an
# Arch Linux docker image.
set -e


####################################################################################################
# Perform image customization inside chroot
#
# What happens in the chroot is entirely driven by the contents of /mnt/staging, which is intended
# to be mounted to a host directory containing staging content for the ARM image. At the very least,
# this directory is expected to contain an executable script named prepare_alarm_image.sh.
####################################################################################################
mnt=/mnt/img

echo "img-build > Setting up chroot"
cp $(which qemu-arm-static) $mnt/usr/bin/qemu-arm-static
mv $mnt/etc/resolv.conf $mnt/etc/resolv.conf.bak
cp /etc/resolv.conf $mnt/etc/resolv.conf
mount --bind /sys $mnt/sys
mount --bind /proc $mnt/proc
mount --bind /dev $mnt/dev
mount --bind /dev/pts $mnt/dev/pts
mkdir $mnt/staging
mount --bind /mnt/staging $mnt/staging

echo "img-build > Performing initialization inside chroot"
chroot $mnt /staging/prepare_alarm_image.sh 2>&1 | sed 's/^/    /'

echo "img-build > Cleaning up chroot"
fuser -k $mnt &> /dev/null || true
echo "foo"
umount $mnt/staging
rmdir $mnt/staging
umount $mnt/dev/pts
umount $mnt/dev
umount $mnt/proc
umount $mnt/sys
mv $mnt/etc/resolv.conf.bak $mnt/etc/resolv.conf
rm $mnt/usr/bin/qemu-arm-static


####################################################################################################
# Convert filesystem contents into SD card image
#
# Although not the most efficient, this approach to packaging the skeleton image does not require
# special kernel support.
#
# Make sure to do some math before changing any of these block sizes.
####################################################################################################
img=/alarm.img

echo "img-build > Creating disk image"
rm -f $img
dd if=/dev/zero of=$img bs=2M count=1024 &> /dev/null
sfdisk $img < /scripts/loop0.sfdisk &> /dev/null

echo "img-build > Populating boot filesystem"
mkfs.fat -F 32 -n BOOT -C /tmp/boot.img 102400 &> /dev/null
mcopy -i /tmp/boot.img -s $mnt/boot/* ::
rm -rf $mnt/boot/*
dd if=/tmp/boot.img of=$img bs=512 seek=2048 conv=notrunc &> /dev/null

echo "img-build > Populating root filesystem"
mkfs.ext4 -b 4096 -d $mnt -L root /tmp/root.img 498432 &> /dev/null
dd if=/tmp/root.img of=$img bs=512 seek=206848 conv=notrunc &> /dev/null

echo "img-build > Flushing disk cache"
sync

echo "img-build > Disk image is ready!"
