#!/bin/bash

# Uses QEMU and chroot to run the specified command within the hosted Arch ARM system. $1 is used as
# the root of the hosted system, $2 is the command to run inside that system, and all remaining args
# are passed through to $2.

set -e
root=$1
cmd=$2
shift 2


# Use QEMU and binfmt_misc to emulate ARM
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
mount --bind /tmp $root/tmp
mkdir -p $root/source
mount --bind /source $root/source


# Do the chroot
echo "alarm-chroot.sh: running command \"$cmd\""
chroot $root $cmd "$@" 2>&1 | sed 's/^/    /'
cmdStatus=${PIPESTATUS[0]}


# Try to kill any processes leftover by the chroot
fuser -sk $root || true

# Clean up mount points
umount $root/source
rmdir $root/source
umount $root/tmp
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
