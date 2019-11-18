#!/bin/bash -e

install -m 755 files/lcsvc         "${ROOTFS_DIR}/usr/bin/lcsvc"
install -m 644 files/lcsvc.service "${ROOTFS_DIR}/usr/lib/systemd/system/lcsvc.service"
install -m 440 files/sudoers       "${ROOTFS_DIR}/etc/sudoers.d/lcsvc"

function install_dir() {
    src=$1
    dst=$2
    mkdir -p $dst
    rsync -r $src $dst
    find $dst -type d -exec chmod 755 {} +
    find $dst -type f -exec chmod 644 {} +
}

install_dir "files/templates/" "${ROOTFS_DIR}/usr/share/lunacam/templates"

on_chroot << EOF
systemctl enable lcsvc
EOF
