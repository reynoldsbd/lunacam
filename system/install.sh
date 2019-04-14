#!/bin/bash

# Installs LunaCam on Arch ARM

set -e

staging="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"


# Install dependencies
pacman --noconfirm -Syu --needed nginx ffmpeg

# Install items from staging/root to the system
files=$(cd $staging/root && find . -type f)
for file in $files
do
    echo "--> installing $staging/root/$file to /$file"
    install -D $staging/root/$file /$file
done


# Configure startup services
systemctl enable systemd-networkd
systemctl enable systemd-resolved
systemctl enable lunacam
systemctl enable lunacam-web

# If provided, also perform local initialization
if [ -d $staging/root.local ]
then
    files=$(cd $staging/root.local && find . -type f)
    for file in $files
    do
        echo "--> installing $staging/root.local/$file to /$file"
        install -D $staging/root.local/$file /$file
    done
fi
if [ -f $staging/local.sh ]
then
    $staging/local.sh
fi
