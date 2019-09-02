# TODO: would be able to use alpine:latest if we could use qemu/binfmt/etc...
FROM archlinux/base

RUN pacman --noconfirm -Syu wget
RUN wget -O /alarm-base.tar.gz http://os.archlinuxarm.org/os/ArchLinuxARM-rpi-latest.tar.gz
RUN (mkdir /img && bsdtar -xpf /alarm-base.tar.gz -C /img) || true

RUN pacman --noconfirm -Syu base-devel git parted qemu qemu-arch-extra dosfstools e2fsprogs psmisc \
    mtools
COPY aur-install.sh /usr/bin/aur-install
RUN aur-install qemu-user-static-bin
