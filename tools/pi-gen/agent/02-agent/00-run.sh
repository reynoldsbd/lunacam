#!/bin/bash -e

install -m 644 files/tmpfiles.conf   "${ROOTFS_DIR}/usr/lib/tmpfiles.d/lcagent.conf"
install -m 644 files/lcagent.service "${ROOTFS_DIR}/usr/lib/systemd/system/lcagent.service"

# lcagent binary is built and placed under files by Makefile
install -m 755 files/lcagent         "${ROOTFS_DIR}/usr/bin/lcagent"

on_chroot << EOF
systemctl enable lcagent
EOF
