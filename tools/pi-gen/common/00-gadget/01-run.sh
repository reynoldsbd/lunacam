#!/bin/bash -e

install -m 644 files/config.txt               "${ROOTFS_DIR}/boot/config.txt"
install -m 644 files/gadget.conf              "${ROOTFS_DIR}/etc/modules-load.d/gadget.conf"
install -m 644 files/usb0.if                  "${ROOTFS_DIR}/etc/network/interfaces.d/usb0"
install -m 644 files/isc-dhcp-server.defaults "${ROOTFS_DIR}/etc/default/isc-dhcp-server"
install -m 664 files/dhcpcd.conf              "${ROOTFS_DIR}/etc/dhcpcd.conf"
install -m 644 files/dhcpd.conf               "${ROOTFS_DIR}/etc/dhcp/dhcpd.conf"

on_chroot << EOF
systemctl enable isc-dhcp-server
EOF
