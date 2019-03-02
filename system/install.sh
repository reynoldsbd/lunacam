#! /bin/bash
#
# Customizes the Arch ARM image
set -e
staging=$1


# Setup USB gadget ethernet
# echo "install.sh > configuring USB gadget ethernet"
# echo "dtoverlay=dwc2" >> /boot/config.txt
# install -D /mnt/root/etc/modules-load.d/raspberrypi.conf /etc/modules-load.d/raspberrypi.conf
# install -D /mnt/root/etc/modprobe.d/g_ether.conf         /etc/modprobe.d/g_ether.conf
# install -D /mnt/root/etc/systemd/network/gadget.network  /etc/systemd/network/gadget.network
# systemctl enable systemd-networkd


# Update system and install dependencies
# TODO: can this be done during docker build?
# echo "install.sh > updating system"
# pacman-key --init
# pacman-key --populate archlinuxarm
# pacman --noconfirm -Syu base-devel git sudo nginx ffmpeg
# pacman --noconfirm -U /mnt/ffmpeg-mmal.pkg.tar.xz


# Install LunaCam
(cd $staging/root && find . -type f) | xargs -i install -D $staging/root/{} /{}
systemctl enable systemd-networkd
systemctl enable lunacam
systemctl enable lunacam-web


exit 0


####################################################################################################
# TODO:
# Running the commented-out section below results in an un-bootable system. Don't fully know the
# root cause, but I think it's because mkinitcpio doesn't like being run in the emulated
# environment.
#
# Possible workarounds:
# * systemd-nspawn
# * full emulation (instead of chroot)
# * no emulation, just pacstrap into the image base from Arch ARM repos
#
# In the meantime, these steps must be run manually after first boot. For convenience, the pre-built
# ffmpeg-mmal package is dropped into /root.
####################################################################################################

####################################################################################################
# TODO:
# Why is this commented out?
####################################################################################################

#echo "alarm-init > Installing LunaCam service"
#install -D /mnt/udev.rules             /usr/lib/udev/rules.d/99-lunacam.conf
#install -D /mnt/lunacam-stream.service /usr/lib/systemd/system/lunacam-stream.service
#systemctl enable lunacam-stream
