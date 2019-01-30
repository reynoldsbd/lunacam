#! /bin/bash
#
# Initializes the environment of the build host

set -e


# Grab any AUR packages required to make this work
# TODO: use separate script to take advantage of docker cache
sudo -u nobody git clone https://aur.archlinux.org/qemu-user-static-bin /tmp/qemu-user-static-bin
(cd /tmp/qemu-user-static-bin && sudo -u nobody makepkg -s)
pacman --noconfirm -U /tmp/qemu-user-static-bin/qemu-user-static-bin-*-x86_64.pkg.tar.xz


# TODO: download base tarball
# TODO: extract somewhere
# TODO: prep for QEMU+chroot
