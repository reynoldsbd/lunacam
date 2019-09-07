#!/bin/bash

# Installs LunaCam Daemon on Arch ARM

set -e

function install_dir {
    files=$(cd $1 && find -type f | cut -c 3-)
    for file in $files
    do
        echo "--> installing $2/$file"
        install -D $1/$file $2/$file
    done
}

echo "install.sh (daemon): installing dependencies"
pacman --noconfirm -Syuq --needed ffmpeg

echo "install.sh (daemon): installing daemon components"
install -D $RUST_OUT_DIR/lunacam-daemon /usr/bin/lunacam-daemon
install_dir sysroot
cat >> /boot/config.txt <<EOL

# Camera module support
start_x=1
gpu_mem=128
EOL

echo "install.sh (daemon): configuring system"
systemctl enable lunacam-daemon

echo "install.sh (daemon): installation complete"
