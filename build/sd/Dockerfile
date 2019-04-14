FROM alpine:latest AS alarmfs
RUN wget -O /alarm-base.tar.gz http://os.archlinuxarm.org/os/ArchLinuxARM-rpi-latest.tar.gz
RUN apk --no-cache add libarchive-tools
RUN (mkdir /img && bsdtar -xpf /alarm-base.tar.gz -C /img) || true

# TODO: would be able to use alpine:latest if we could use qemu/binfmt/etc...
FROM archlinux/base
COPY --from=alarmfs /img /img
RUN pacman --noconfirm -Syu base-devel git wget parted qemu qemu-arch-extra dosfstools e2fsprogs \
    psmisc mtools
COPY scripts/aur-install.sh /scripts/aur-install.sh
RUN /scripts/aur-install.sh qemu-user-static-bin

COPY scripts/ /scripts/
CMD ["/scripts/build.sh"]
