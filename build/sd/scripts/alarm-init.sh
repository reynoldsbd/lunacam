#!/bin/bash

# Performs one-time initialization of Arch ARM image

set -e


pacman-key --init 2>/dev/null 1>/dev/null
pacman-key --populate archlinuxarm 2>/dev/null 1>/dev/null
pacman --noconfirm -Syu --needed sudo
