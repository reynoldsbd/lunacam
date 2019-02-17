#! /bin/bash
#
# Uses QEMU and chroot to run the specified script within the hosted Arch ARM system.
#
# This script expects two positional arguments. $1 must be the path to the root of the Arch ARM's
# extracted filesystem, and $2 is path of the script or binary to use with chroot. Note that you may
# omit $2 to start an interactive shell.
#
# For convenience, the /artifacts directory of the  Docker image is mounted within the chroot at
# /mnt.
set -e
root=$1
cmd=$2


# Use QEMU and binfmt_misc for emulating ARM
cp $(which qemu-arm-static) $root/usr/bin/qemu-arm-static

# Use the host's resolv.conf to enable DNS resolution from the hosted system
if [ ! -f $root/etc/resolv.conf.bak ]; then
    mv $root/etc/resolv.conf $root/etc/resolv.conf.bak
fi
cp /etc/resolv.conf $root/etc/resolv.conf

# Setup mount points needed for a working Arch environment
mount --bind $root $root
mount --bind $root/boot $root/boot
mount --bind /sys $root/sys
mount --bind /proc $root/proc
mount --bind /dev $root/dev
mount --bind /dev/pts $root/dev/pts
# mkdir -p $root/scripts
mount --bind /artifacts $root/mnt
# mount --bind /mnt $root/mnt


# Do the chroot
chroot $root $cmd 2>&1 | sed 's/^/    /'
cmdStatus=${PIPESTATUS[0]}


# Try to kill any processes leftover by the chroot
fuser -sk $root || true

# Clean up mount points
umount $root/mnt
# umount $root/scripts
# rmdir $root/scripts
umount $root/dev/pts
umount $root/dev
umount $root/proc
umount $root/sys
umount $root/boot
umount $root

# Restore original resolv.conf
mv $root/etc/resolv.conf.bak $root/etc/resolv.conf

# Cleanup QEMU interpreter
rm $root/usr/bin/qemu-arm-static

# Pass status code from cmd back to caller
exit $cmdStatus
