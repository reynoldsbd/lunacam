#!/bin/bash -e
source "${LC_UTILS}"

install -m 755 files/lcsvc         "${ROOTFS_DIR}/usr/bin/lcsvc"
install -m 644 files/lcsvc.service "${ROOTFS_DIR}/usr/lib/systemd/system/lcsvc.service"
install -m 440 files/sudoers       "${ROOTFS_DIR}/etc/sudoers.d/lcsvc"

install_dir "files/templates/" "${ROOTFS_DIR}/usr/share/lunacam/templates"

on_chroot << EOF
systemctl enable lcsvc
EOF
