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


FROM rust_base AS build

ENTRYPOINT ["./run.sh"]

CMD ["build"]

FROM build AS release_build

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
COPY Cargo.lock .
COPY Cargo.toml .

RUN chown -R $USERNAME:$USERNAME /build
# Create the database file
RUN sudo -Hiu $USERNAME bash -c 'cd /build && DATABASE_URL=sqlite:///build/hare.db \$HOME/.cargo/bin/cargo sqlx database setup --source src/hare-data-plane-client/migrations/ --sqlite-create-db-wal false'
# Compile the server
RUN sudo -Hiu $USERNAME bash -c 'cd /build && DATABASE_URL=sqlite:///build/hare.db \$HOME/.cargo/bin/cargo build --profile release-fat-lto --target x86_64-unknown-linux-musl -p hare-control-plane-server'
# Create app directory with server and database file owned by run-as user
RUN mkdir app
RUN cp /build/target/x86_64-unknown-linux-musl/release-fat-lto/hare-control-plane-server app/hare-control-plane-server
RUN chmod 555 app/hare-control-plane-server
RUN cp /build/hare.db app/hare.db
RUN chmod 644 app/hare.db
RUN chown -R $USERNAME:$USERNAME app/

FROM scratch AS release

COPY --from=release_build /etc/passwd /etc/passwd
COPY --from=release_build /etc/group /etc/group

COPY --from=release_build --chown=hare:hare /build/app/ /app/

WORKDIR /app

USER hare:hare

ENV DATABASE_URL="/app/hare.db"
ENV RUST_LOG=debug,h2=info,sqlx=info

ENTRYPOINT [ "/app/hare-control-plane-server" ]
CMD [ "0.0.0.0:8000" ]