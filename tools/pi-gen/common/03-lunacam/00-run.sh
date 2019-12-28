#!/bin/bash -e
source "${LC_UTILS}"

install -m 755 files/lunacam         "${ROOTFS_DIR}/usr/bin/lunacam"
install -m 644 files/lunacam.service "${ROOTFS_DIR}/usr/lib/systemd/system/lunacam.service"
install -m 440 files/sudoers         "${ROOTFS_DIR}/etc/sudoers.d/lunacam"

install_dir "files/templates/" "${ROOTFS_DIR}/usr/share/lunacam/templates"

on_chroot << EOF
systemctl enable lunacam
EOF
