#! /bin/bash
#
# Customizes the Arch ARM image
set -e


# Setup USB gadget ethernet
echo "install.sh > configuring USB gadget ethernet"
echo "dtoverlay=dwc2" >> /boot/config.txt
install -D /mnt/root/etc/modules-load.d/raspberrypi.conf /etc/modules-load.d/raspberrypi.conf
install -D /mnt/root/etc/modprobe.d/g_ether.conf         /etc/modprobe.d/g_ether.conf
install -D /mnt/root/etc/systemd/network/gadget.network  /etc/systemd/network/gadget.network
systemctl enable systemd-networkd


# Update system and install dependencies
echo "install.sh > updating system"
pacman-key --init
pacman-key --populate archlinuxarm
pacman --noconfirm -Syu base-devel git sudo nginx
# pacman --noconfirm -U /mnt/ffmpeg-mmal.pkg.tar.xz


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

#echo "alarm-init > Updating and installing packages"
#pacman-key --init
#pacman-key --populate archlinuxarm
#pacman --noconfirm -Syu base-devel git sudo nginx
#pacman --noconfirm -U /mnt/ffmpeg-mmal.pkg.tar.xz
cp /mnt/ffmpeg-mmal.pkg.tar.xz /root/ffmpeg-mmal.pkg.tar.xz

echo "alarm-init > Configuring USB OTG ethernet access"
echo "dtoverlay=dwc2" >> /boot/config.txt
cp /mnt/raspberrypi.conf /etc/modules-load.d/raspberrypi.conf
cp /mnt/g_ether.conf     /etc/modprobe.d/g_ether.conf
cp /mnt/gadget.network   /etc/systemd/network/gadget.network
systemctl enable systemd-networkd

####################################################################################################
# TODO:
# Why is this commented out?
####################################################################################################

#echo "alarm-init > Installing LunaCam service"
#install -D /mnt/nginx.conf             /usr/share/lunacam/nginx.conf
#install -D /mnt/index.html             /usr/share/lunacam/http/index.html
#install -D /mnt/udev.rules             /usr/lib/udev/rules.d/99-lunacam.conf
#install -D /mnt/sysusers.conf          /usr/lib/sysusers.d/lunacam.conf
#install -D /mnt/tmpfiles.conf          /usr/lib/tmpfiles.d/lunacam.conf
#install -D /mnt/lunacam-web.service    /usr/lib/systemd/system/lunacam-web.service
#install -D /mnt/lunacam-stream.service /usr/lib/systemd/system/lunacam-stream.service
#systemctl enable lunacam-web
#systemctl enable lunacam-stream
