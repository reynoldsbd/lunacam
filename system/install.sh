#! /bin/bash
#
# Customizes the Arch ARM image
set -e
staging=$1


# Update system and install dependencies
# TODO: this shouldn't be part of install.sh
# pacman-key --init 2>/dev/null 1>/dev/null
# pacman-key --populate archlinuxarm 2>/dev/null 1>/dev/null
# pacman --noconfirm -Syu --needed sudo nginx ffmpeg

# Install items from staging/root to the system
files=$(cd $staging/root && find . -type f)
for file in $files
do
    echo "--> installing $staging/root/$file to /$file"
    install -D $staging/root/$file /$file
done

# If provided, also install items from root.local
if [ -d $staging/root.local ]
then
    files=$(cd $staging/root.local && find . -type f)
    for file in $files
    do
        echo "--> installing $staging/root.local/$file to /$file"
        install -D $staging/root.local/$file /$file
    done
fi


# Configure startup services
systemctl enable systemd-networkd
systemctl enable lunacam
systemctl enable lunacam-web
