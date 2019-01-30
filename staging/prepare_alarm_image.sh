#! /bin/bash
#
# This script customizes an Arch ARM image. It is intended to be run *inside* the new image, using
# a chroot.
set -e

echo "Updating system packages"
# pacman-key --init
# pacman-key --populate archlinuxarm
# pacman -Syu

echo "Configuring USB OTG network access"
echo "dtoverlay=dwc2" >> /boot/config.txt
cp /staging/raspberrypi.conf /etc/modules-load.d/raspberrypi.conf
cp /staging/g_ether.conf /etc/modprobe.d/g_ether.conf
cp /staging/gadget.network /etc/systemd/network/gadget.network
systemctl enable systemd-networkd
