# The image created by this Dockerfile provides an environment for creating Arch ARM images.

FROM archlinux/base

# Install dependencies
# TODO: clean up this list after rewrite
RUN pacman --noconfirm -Syu base-devel git wget parted qemu qemu-arch-extra dosfstools e2fsprogs \
    psmisc mtools
# TODO: is there something I can install via pacman to make this simpler?
COPY install_aur_packages.sh /scripts/install_aur_packages.sh
RUN ["/scripts/install_aur_packages.sh"]

# Start with the base filesystem from Arch ARM
RUN wget -O /arch-arm-base.tar.gz http://os.archlinuxarm.org/os/ArchLinuxARM-rpi-latest.tar.gz
COPY prepare_root_filesystem.sh /scripts/prepare_root_filesystem.sh
RUN ["/scripts/prepare_root_filesystem.sh"]

COPY loop0.sfdisk /scripts/loop0.sfdisk
COPY build_alarm_image.sh /scripts/build_alarm_image.sh

CMD ["/scripts/build_alarm_image.sh"]
