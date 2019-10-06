#!/bin/bash -e

install -m 644 files/sysusers.conf "${ROOTFS_DIR}/usr/lib/sysusers.d/lunacam.conf"
