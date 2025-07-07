#!/usr/bin/env bash

set -e

. /build-support/shell/common/log.sh


if [ -z "${HARE_COMPONENT}" ]
then
    error "Must set HARE_COMPONENT environment variable"
    exit 1
fi

if [ -z "${USERNAME}" ]
then
    error "Must set USERNAME environment variable"
    exit 1
fi

mkdir app

# Install the MUSL toolchain
apt install -y musl-tools musl-dev

# Detect the host CPU architecture to match the build target
TARGET="$(uname -p)-unknown-linux-musl"
# Add the MUSL Rust target
sudo -Hiu $USERNAME bash -c "\$HOME/.cargo/bin/rustup target add ${TARGET}"

if [ "${HARE_COMPONENT}" = "server" ]
then
    if [ -z "${HARE_BACKEND}" ]
    then
        error "Must set HARE_BACKEND environment variable"
        exit 1
    fi

    # Compile the control plane server
    sudo -Hiu $USERNAME bash -c "cd /build && \$HOME/.cargo/bin/cargo build --profile release-fat-lto --target ${TARGET} -p hare-control-plane-server --no-default-features --features ${HARE_BACKEND}"
    # Compile the web server
    sudo -Hiu $USERNAME bash -c "cd /build && \$HOME/.cargo/bin/cargo build --profile release-fat-lto --target ${TARGET} -p hare-web-server --no-default-features --features ${HARE_BACKEND}"

    # Copy server executables to app directory
    cp /build/target/${TARGET}/release-fat-lto/hare-control-plane-server app/hare-control-plane-server
    cp /build/target/${TARGET}/release-fat-lto/hare-web-server app/hare-web-server
elif [ "${HARE_COMPONENT}" = "cli" ]
then
    sudo -Hiu $USERNAME bash -c "cd /build && \$HOME/.cargo/bin/cargo build --profile release-fat-lto --target ${TARGET} -p hare-cli"
    cp /build/target/${TARGET}/release-fat-lto/hare-cli app/hare-cli
else
    error "Unknown component ${HARE_COMPONENT}"
    exit 1
fi

# Strip the build artifacts
strip app/hare-*
# Make the build artifacts read/execute-only
chmod 555 app/hare-*