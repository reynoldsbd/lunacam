#!/bin/bash

# Installs LunaCam Daemon on Arch ARM

set -e
. $LC_TOOLS/imagebuild/install-helpers.sh

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

if [ -d sysroot.local ]
then
    echo "install.sh (daemon): installing local sysroot"
    install_dir sysroot.local
fi

if [ -f install.local.sh ]
then
    echo "install.sh (daemon): running local install script"
    ./install.local.sh
fi

echo "install.sh (daemon): installation complete"
