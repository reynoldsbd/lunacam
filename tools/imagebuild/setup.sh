#!/bin/bash

# Sets up an Arch ARM image for use with LunaCam

set -e
. /source/tools/imagebuild/install-helpers.sh
variant=$1

if [ -f /source/sysroot.local/etc/pacman.d/mirrorlist ]
then
    echo "setup.sh: using custom mirrorlist"
    cp /source/sysroot.local/etc/pacman.d/mirrorlist /etc/pacman.d/mirrorlist
fi

echo "setup.sh: initializing pacman"
pacman-key --init 2>/dev/null 1>/dev/null
pacman-key --populate archlinuxarm 2>/dev/null 1>/dev/null

echo "setup.sh: installing common dependencies"
pacman --noconfirm -Syuq --needed sudo base-devel nginx

echo "setup.sh: disabling root user"
usermod -p '!' root

echo "setup.sh: deleting alarm user"
if id -u alarm &> /dev/null
then
    userdel -r alarm
fi

echo "setup.sh: installing sysroot"
install_dir /source/tools/imagebuild/sysroot

echo "setup.sh: enabling default services"
systemctl enable systemd-networkd
systemctl enable systemd-resolved
systemctl enable nginx

echo "setup.sh: installing $variant"
export RUST_TARGET=arm-unknown-linux-gnueabihf
export RUST_PROFILE=release
make --no-print-directory -C /source install-$variant

if [ -d /source/sysroot.local ]
then
    echo "setup.sh: installing local sysroot"
    install_dir /source/sysroot.local
fi

if [ -f /source/install.local.sh ]
then
    echo "setup.sh: running local install script"
    /source/install.local.sh
fi

echo "setup.sh: setup complete"
