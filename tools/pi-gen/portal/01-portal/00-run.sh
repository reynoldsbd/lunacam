#!/bin/bash -e

install -m 644 files/lcportal.service "${ROOTFS_DIR}/usr/lib/systemd/system/lcportal.service"
install -m 440 files/sudoers          "${ROOTFS_DIR}/etc/sudoers.d/lcportal"

# lcportal binary is built and placed under files by Makefile
install -m 755 files/lcportal         "${ROOTFS_DIR}/usr/bin/lcportal"

function install_dir() {
    src=$1
    dst=$2
    mkdir -p $dst
    rsync -r $src $dst
    find $dst -type d -exec chmod 755 {} +
    find $dst -type f -exec chmod 644 {} +
}

install_dir "files/static/"    "${ROOTFS_DIR}/usr/share/lunacam/static"
install_dir "files/templates/" "${ROOTFS_DIR}/usr/share/lunacam/templates"

on_chroot << EOF
systemctl enable lcportal
EOF
