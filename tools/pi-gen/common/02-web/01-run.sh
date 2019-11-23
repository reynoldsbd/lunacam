#!/bin/bash -e

install -m 644 files/nginx.conf "${ROOTFS_DIR}/etc/nginx/nginx.conf"

on_chroot << EOF
systemctl enable nginx
EOF
