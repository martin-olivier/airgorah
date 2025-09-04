FROM rust:1.88.0-slim-bookworm

# Fetch package list
RUN apt update

# Install build dependencies
RUN apt install -y build-essential libgtk-4-dev libglib2.0-dev

# Install packaging tools
RUN apt install -y ruby ruby-dev rubygems rpm zstd libarchive-tools

# Install fpm
RUN gem install fpm

# Install rustup components
RUN rustup component add clippy
RUN rustup component add rustfmt

##### Commands #####

WORKDIR /workspace

ENV DEBIAN_DEPS="--depends libgtk-4-1 --depends dbus-x11 --depends wireshark-common --depends iproute2 --depends mdk4 --depends crunch"
ENV REDHAT_DEPS="--depends gtk4-devel --depends dbus-x11 --depends wireshark-cli --depends iproute"
ENV ARCHLINUX_DEPS="--depends gtk4 --depends dbus --depends wireshark-cli --depends iproute2 --depends mdk4"

# Build and package the project
CMD cargo build && \
    fpm -f -t deb -p airgorah_`arch`.deb -a native $DEBIAN_DEPS && \
    fpm -f -t rpm -p airgorah_`arch`.rpm -a native $REDHAT_DEPS && \
    fpm -f -t pacman -p airgorah_`arch`.pkg.tar.zst -a native $ARCHLINUX_DEPS
