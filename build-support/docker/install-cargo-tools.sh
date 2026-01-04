#!/usr/bin/env bash

set -e

. /build-support/shell/common/log.sh


if [ -z "${USERNAME}" ]
then
    error "Must set USERNAME environment variable"
    exit 1
fi

## NOTE: Temporarily disabled becuase not being used
## info "Installing Cross for cross-compilation and testing"
## sudo -Hiu $USERNAME bash -c '$HOME/.cargo/bin/cargo install --locked --version 0.2.1 cross'
## 
## info "Installing cargo-deny for dependency linting"
## sudo -Hiu $USERNAME bash -c '$HOME/.cargo/bin/cargo install --locked cargo-deny'

info "Installing llvm-cov for creating coverage reports"
sudo -Hiu $USERNAME bash -c '$HOME/.cargo/bin/cargo install --locked cargo-llvm-cov'

info "Installing sqlx CLI for managing the database"
sudo -Hiu $USERNAME bash -c '$HOME/.cargo/bin/cargo install --locked sqlx-cli'
