FROM archlinux/base

WORKDIR /root

# RUN apk add --no-cache curl git
RUN pacman --noconfirm -Sy git

# Install and configure Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
ENV PATH /root/.cargo/bin:$PATH
RUN rustup target add arm-unknown-linux-gnueabihf

# Download the official Raspberry Pi cross-compiler
RUN git clone --depth 1 https://github.com/raspberrypi/tools /root/rpi-tools
ENV PATH /root/rpi-tools/arm-bcm2708/arm-linux-gnueabihf/bin:$PATH
