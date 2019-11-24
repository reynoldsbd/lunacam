#!/bin/bash -e

function install_dir() {
    src=$1
    dst=$2
    mkdir -p $dst
    rsync -r $src $dst
    find $dst -type d -exec chmod 755 {} +
    find $dst -type f -exec chmod 644 {} +
}

install_dir "files/static/"    "${ROOTFS_DIR}/usr/share/lunacam/static"
