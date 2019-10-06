#!/bin/bash -e

install -m 644 files/lcportal.service "${ROOTFS_DIR}/usr/lib/systemd/system/lcportal.service"
install -m 644 files/lcportal         "${ROOTFS_DIR}/usr/bin/lcportal"
install -m 440 files/sudoers          "${ROOTFS_DIR}/etc/sudoers.d/lcportal"

rsync -r files/static/    "${ROOTFS_DIR}/usr/share/lunacam/static"
chmod -R 644              "${ROOTFS_DIR}/usr/share/lunacam/static"

rsync -r files/templates/ "${ROOTFS_DIR}/usr/share/lunacam/templates"
chmod -R 644              "${ROOTFS_DIR}/usr/share/lunacam/templates"

on_chroot << EOF
systemctl enable lcportal
EOF
