# The image created by this Dockerfile provides an environment for creating Arch ARM images.

FROM archlinux/base

# Install dependencies
# TODO: clean up this list after rewrite
RUN pacman --noconfirm -Syu base-devel git wget parted qemu qemu-arch-extra dosfstools e2fsprogs \
    psmisc mtools
COPY aur-install.sh /scripts/aur-install.sh
RUN /scripts/aur-install.sh qemu-user-static-bin

# Start with the base filesystem from Arch ARM
RUN wget -O /alarm-base.tar.gz http://os.archlinuxarm.org/os/ArchLinuxARM-rpi-latest.tar.gz
RUN (mkdir /img && bsdtar -xpf /alarm-base.tar.gz -C /img) || true

COPY alarm-chroot.sh /scripts/alarm-chroot.sh
COPY loop0.sfdisk /scripts/loop0.sfdisk
COPY alarm-build.sh /scripts/alarm-build.sh
COPY alarm-init.sh /scripts/alarm-init.sh
CMD /scripts/alarm-build.sh
