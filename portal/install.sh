#!/bin/sh

# Installs LunaCam Portal on Arch ARM

set -e
. $LC_TOOLS/imagebuild/install-helpers.sh

echo "install.sh (portal): installing portal components"
install -D $RUST_OUT_DIR/lunacam-portal /usr/bin/lunacam-portal
install_dir sysroot
install_dir $STATIC_DIR /usr/share/lunacam/static
install_dir templates /usr/share/lunacam/templates

echo "install.sh (portal): configuring system"
systemctl enable lunacam-portal

if [ -d sysroot.local ]
then
    echo "install.sh (portal): installing local sysroot"
    install_dir sysroot.local
fi

if [ -f install.local.sh ]
then
    echo "install.sh (portal): running local install script"
    ./install.local.sh
fi

echo "install.sh (portal): installation complete"
