# Base, language-agnostic image with Bash shell and new user
FROM ubuntu:jammy AS base

ARG UID
ARG USERNAME

# Update base packages
RUN apt update && \
    apt upgrade -y

# Add Bash
# Used as shell for new user.
RUN apt install -y bash

# Copy Docker build scripts
COPY build-support/shell/common/log.sh /build-support/shell/common/log.sh
COPY build-support/shell/run/config.sh /build-support/shell/run/config.sh

# Create new user
# This user is used to run all user-specified commands
# in the container, and the UID/USERNAME should match
# the local (host) user to ensure file permissions of generated
# or touched files are correct.
COPY build-support/docker/adduser.sh /build-support/docker/adduser.sh
RUN UID=$UID \
    USERNAME=$USERNAME \
    /build-support/docker/adduser.sh


# Base image for using Rustup and Rust tools
FROM base AS rust_base

# GID of the local Docker group
ARG DOCKER_GID
# Rust version installed by Rustup
ARG RUST_VERSION=stable
# Must match USERNAME in base image
ARG USERNAME
# Let cross know it's running in a container
# See: https://github.com/cross-rs/cross#docker-in-docker
ENV CROSS_DOCKER_IN_DOCKER=true

COPY build-support/docker/install-docker.sh /build-support/docker/install-docker.sh
RUN DOCKER_GID=$DOCKER_GID \
    /build-support/docker/install-docker.sh

COPY build-support/docker/install-rust.sh /build-support/docker/install-rust.sh
RUN USERNAME=$USERNAME \
    RUST_VERSION=$RUST_VERSION \
    /build-support/docker/install-rust.sh

COPY build-support/docker/install-cargo-tools.sh /build-support/docker/install-cargo-tools.sh
RUN USERNAME=$USERNAME \
    /build-support/docker/install-cargo-tools.sh

COPY build-support/docker/install-protobuf-tools.sh /build-support/docker/install-protobuf-tools.sh
RUN /build-support/docker/install-protobuf-tools.sh

COPY build-support/docker/install-cdk-tools.sh /build-support/docker/install-cdk-tools.sh
RUN /build-support/docker/install-cdk-tools.sh


FROM rust_base AS build

ENTRYPOINT ["./run.sh"]

CMD ["build"]

FROM build AS release_build

# Which database backend to use
ARG HARE_BACKEND
# Which component to build image for
ARG HARE_COMPONENT

# Create dedicated run-as user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home /nonexistent \
    --shell /sbin/nologin \
    --no-create-home \
    --uid 10001 \
    hare

WORKDIR /build

COPY ./proto ./proto
COPY ./src ./src
COPY ./.cargo ./.cargo
COPY Cargo.lock .
COPY Cargo.toml .

RUN chown -R $USERNAME:$USERNAME /build
COPY build-support/docker/build-release.sh /build-support/docker/build-release.sh
RUN HARE_BACKEND=$HARE_BACKEND \
    HARE_COMPONENT=$HARE_COMPONENT \
    USERNAME=$USERNAME \
    /build-support/docker/build-release.sh
RUN chown -R $USERNAME:$USERNAME app/

FROM alpine:latest AS release

COPY --from=release_build /etc/passwd /etc/passwd
COPY --from=release_build /etc/group /etc/group

COPY --from=release_build --chown=hare:hare /build/app/ /app/

# Install curl for health checks
RUN apk add --no-cache curl

WORKDIR /app

USER hare:hare
ENV RUST_LOG=debug,h2=info,sqlx=info