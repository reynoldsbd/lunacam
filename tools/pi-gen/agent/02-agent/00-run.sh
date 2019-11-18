#!/bin/bash -e

install -m 644 files/tmpfiles.conf "${ROOTFS_DIR}/usr/lib/tmpfiles.d/lcagent.conf"
