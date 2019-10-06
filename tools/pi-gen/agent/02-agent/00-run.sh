#!/bin/bash -e

install -m 644 files/tmpfiles.conf   "${ROOTFS_DIR}/usr/lib/tmpfiles.d/lcagent.conf"
install -m 644 files/lcagent.service "${ROOTFS_DIR}/usr/lib/systemd/system/lcagent.service"
install -m 644 files/lcagent         "${ROOTFS_DIR}/usr/bin/lcagent"

on_chroot << EOF
systemctl enable lcagent
EOF
