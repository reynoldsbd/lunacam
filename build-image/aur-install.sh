#! /bin/bash
#
# Installs the specified AUR packages
set -e


for pkg; do
    sudo -u nobody git clone https://aur.archlinux.org/$pkg /tmp/$pkg > /dev/null
    (cd /tmp/$pkg && sudo -u nobody makepkg -s > /dev/null)
    pacman --noconfirm -U /tmp/$pkg/$pkg-*.pkg.tar.xz
done
