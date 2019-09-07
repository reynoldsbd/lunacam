#!/bin/sh

# Installs LunaCam Portal on Arch ARM

set -e

function install_dir {
    files=$(cd $1 && find -type f | cut -c 3-)
    for file in $files
    do
        echo "--> installing $2/$file"
        install -D $1/$file $2/$file
    done
}

echo "install.sh (portal): installing portal components"
install -D $RUST_OUT_DIR/lunacam-portal /usr/bin/lunacam-portal
install_dir sysroot
install_dir $STATIC_DIR /usr/share/lunacam/static
install_dir templates /usr/share/lunacam/templates

echo "install.sh (portal): configuring system"
systemctl enable lunacam-portal

echo "install.sh (portal): installation complete"
