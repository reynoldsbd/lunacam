#!/bin/bash -e

install -m 644 files/modules-load.conf "${ROOTFS_DIR}/usr/lib/modules-load.d/lcagent.conf"
install -m 644 files/udev.rules        "${ROOTFS_DIR}/usr/lib/udev/rules.d/lcagent.rules"

cat >> "${ROOTFS_DIR}/boot/config.txt" << EOF

# Camera module support
start_x=1
gpu_mem=128
EOF
