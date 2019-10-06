#!/bin/bash -e

on_chroot << EOF
systemctl enable nginx
EOF
