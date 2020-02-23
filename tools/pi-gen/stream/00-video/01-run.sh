#!/bin/bash -e

install -m 644    files/modules-load.conf   "${ROOTFS_DIR}/usr/lib/modules-load.d/lunacam-stream.conf"
install -m 644    files/udev.rules          "${ROOTFS_DIR}/usr/lib/udev/rules.d/lunacam-stream.rules"
install -m 644 -D files/lunacam-reload.conf "${ROOTFS_DIR}/usr/lib/systemd/system/lunacam.service.d/reload.conf"
install -m 644    files/tmpfiles.conf       "${ROOTFS_DIR}/usr/lib/tmpfiles.d/lunacam-stream.conf"

cat >> "${ROOTFS_DIR}/boot/config.txt" << EOF

# Camera module support
start_x=1
gpu_mem=128
EOF
