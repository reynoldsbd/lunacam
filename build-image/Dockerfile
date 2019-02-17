FROM archlinux/base

# Install dependencies
# TODO: clean up this list after rewrite
RUN pacman --noconfirm -Syu base-devel git wget parted qemu qemu-arch-extra dosfstools e2fsprogs \
    psmisc mtools

# Extract the base Arch ARM system
RUN wget -O /alarm-base.tar.gz http://os.archlinuxarm.org/os/ArchLinuxARM-rpi-latest.tar.gz
RUN (mkdir /img && bsdtar -xpf /alarm-base.tar.gz -C /img) || true

COPY aur-install.sh /root/aur-install.sh
RUN /root/aur-install.sh qemu-user-static-bin

VOLUME /artifacts

COPY loop0.sfdisk /root/loop0.sfdisk
COPY alarm-chroot.sh /root/alarm-chroot.sh
COPY build.sh /root/build.sh
CMD ["/root/build.sh"]
