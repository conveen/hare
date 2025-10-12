#!/usr/bin/env bash

set -e

###################
##### Imports #####
###################

# Check if running in build container or locally
# to import from correct path
if [ -d /build-support ]
then
    . ${HOME}/.bashrc
    . ${HOME}/.cargo/env
    BUILD_SUPPORT_ROOT="/build-support"
else
    BUILD_SUPPORT_ROOT="./build-support"
fi
. "${BUILD_SUPPORT_ROOT}/shell/run/config.sh"
. "${BUILD_SUPPORT_ROOT}/shell/common/log.sh"


###############################
##### Container Utilities #####
###############################

run-build-base() {
    ${CONTAINER_RUNTIME} build \
        --target "${BUILD_TARGET_STAGE}" \
        -t "${BUILD_IMAGE_URL}:${BUILD_IMAGE_TAG}" \
        -f build-support/docker/Dockerfile \
        --build-arg DOCKER_GID="${DOCKER_GID}" \
        --build-arg RUST_VERSION="${DEFAULT_RUST_VERSION}" \
        --build-arg UID="${USERID}" \
        --build-arg USERNAME="${USERNAME}" \
        "${@}" \
        .
}

run-push-base() {
    ${CONTAINER_RUNTIME} push \
        "${@}" \
        "${BUILD_IMAGE_URL}:${BUILD_IMAGE_TAG}"
}

run-in-container() {
    local COMMAND="${1}"
    local DATABASE_URL="${DATABASE_URL:-sqlite:///project/src/hare-data-plane-client/hare.db}"
    # If input device is not a TTY don't run with `-it` flags
    local INTERACTIVE_FLAGS="$(test -t 0 && echo '-it' || echo '')"
    # Expose ports on localhost for specific commands
    local PORT_FLAGS=""
    if [ "${COMMAND}" = "run-cp" ]
    then
        PORT_FLAGS="-p 127.0.0.1:5001:5001"
    elif [ "${COMMAND}" = "run-web" ]
    then
        PORT_FLAGS="-p 127.0.0.1:8001:8001"
    fi
    ${CONTAINER_RUNTIME} run \
		--rm \
         ${INTERACTIVE_FLAGS} \
         ${PORT_FLAGS} \
		-u ${USERNAME} \
        -e "CROSS_CONTAINER_IN_CONTAINER=true" \
        -e "DATABASE_URL=${DATABASE_URL}" \
        -e "RUST_BACKTRACE" \
        -e "RUST_LOG" \
        -v ${HOME}/.aws:/home/${USERNAME}/.aws \
        -v ${HOME}/.cargo/git:/home/${USERNAME}/.cargo/git \
        -v ${HOME}/.cargo/registry:/home/${USERNAME}/.cargo/registry \
        -v /var/run/docker.sock:/var/run/docker.sock \
		-v $(pwd):/project \
		-w /project \
		${BUILD_IMAGE_URL}:${BUILD_IMAGE_TAG} \
        --local "${@}"
}

run-build-release() {
    local ARCH=${ARCH:-amd64}
    local BACKEND=${BACKEND:-sqlite}
    local COMPONENT=${COMPONENT:-server}
    local TAG
    if [ ${COMPONENT} = "server" ]
    then
        TAG="${RELEASE_IMAGE_URL}/${COMPONENT}/${BACKEND}/${ARCH}:${RELEASE_IMAGE_TAG}"
    else
        TAG="${RELEASE_IMAGE_URL}/${COMPONENT}/${ARCH}:${RELEASE_IMAGE_TAG}"
    fi
    ${CONTAINER_RUNTIME} buildx build \
        --platform "linux/${ARCH}" \
        --target "${RELEASE_TARGET_STAGE}" \
        -t "${TAG}" \
        -f build-support/docker/Dockerfile \
        --build-arg DOCKER_GID="${DOCKER_GID}" \
        --build-arg HARE_BACKEND="${BACKEND}" \
        --build-arg HARE_COMPONENT="${COMPONENT}" \
        --build-arg RUST_VERSION="${DEFAULT_RUST_VERSION}" \
        --build-arg UID="${USERID}" \
        --build-arg USERNAME="${USERNAME}" \
        "${@}" \
        .
}


#############################
##### Command Utilities #####
#############################

# Remove `--release` or `--profile <profile>` from command line flags
remove-profile-flags() {
    local NEW_ARGS="${@}"
    if ( echo "${@}" | grep '\-\-release' 1>/dev/null )
    then
        NEW_ARGS="${NEW_ARGS/--release/}"
    elif ( echo "${@}" | grep '\-\-profile' 1>/dev/null )
    then
        NEW_ARGS="$(echo ${NEW_ARGS} | sed 's/--profile [^ ]\+//g')"
    fi
    echo "${NEW_ARGS}"
}

run-command() {
    local COMMAND="${1}"
    shift

    if [ ${RUNTIME_CONTEXT} = "container" ]
    then
        run-in-container "${COMMAND}" "${@}"
    elif [ ${RUNTIME_CONTEXT} = "local" ]
    then
        run-${COMMAND} "${@}"
    else
        error "Invalid value for RUNTIME_CONTEXT: ${RUNTIME_CONTEXT}"
        exit 1
    fi
}


####################
##### Commands #####
####################

run-build() {
    CHECK_TEST_ARGS="$(remove-profile-flags ${@})"

    run-check ${CHECK_TEST_ARGS}

    run-fmt-check

    run-lint ${CHECK_TEST_ARGS}

    # run-check-deps

    run-test ${CHECK_TEST_ARGS}

    # Clean compilation directory after compiling for tests
    run-clean ${CHECK_TEST_ARGS}

    info "Compiling package"
    cargo build "${@}"
}

run-cdk() {
    info "Running CDK command"
    cd deploy/cdk
    CDK_DISABLE_VERSION_CHECK=1 ${HOME}/.deno/bin/cdk ${@}
}

run-check() {
    info "Checking package for errors"
    # TODO: Fix Cross so it uses the right image and has protoc installed
    cargo check "${@}"
}

run-check-deps() {
    info "Checking dependencies for license compliance"
    cargo deny check licenses "${@}"

    info "Checking dependencies for security notices"
    cargo deny check advisories "${@}"

    info "Checking dependencies for trusted and banned sources"
    cargo deny check bans "${@}" && \
        cargo deny check sources "${@}"
}

run-clean() {
    info "Removing Cargo build artifacts"
    cargo clean "${@}"
}

run-exec() {
    info "Running command: ${*}"
    ${@}
}

run-fmt() {
    info "Formatting code with Rustfmt"
    cargo fmt "${@}"
}

run-fmt-check() {
    info "Checking code format with Rustfmt"
    cargo fmt "${@}" -- --check
}

run-init() {
    if ! [ -z "$(ls src/)" ]
    then
        error "Project already initialized, aborting"
        exit 1
    fi

    read -e -p "Do you want to include .gitconfig in this project's Git config [y/n]? " INCLUDE_GITCONFIG
    if ( [ "${INCLUDE_GITCONFIG,,}" = "y" ] && [ -d "./git/" ] )
    then
        git config --local include.path ../.gitconfig
    fi

    local DEFAULT_PACKAGE_NAME="$(basename $(pwd))"
    local DEFAULT_PACKAGE_TARGET="lib"
    
    read -e -p "Package name [${DEFAULT_PACKAGE_NAME}]: " PACKAGE_NAME
    PACKAGE_NAME="${PACKAGE_NAME:-${DEFAULT_PACKAGE_NAME}}"
    local PACKAGE_DIRECTORY="${PACKAGE_NAME/_/-}"
    read -e -p "Package target (bin or lib) [${DEFAULT_PACKAGE_TARGET}]: " PACKAGE_TARGET
    PACKAGE_TARGET="${PACKAGE_TARGET:-${DEFAULT_PACKAGE_TARGET}}"
    PACKAGE_TARGET="${PACKAGE_TARGET,,}"
    if ! ( [ "${PACKAGE_TARGET}" = "bin" ] || [ "${PACKAGE_TARGET}" = "lib" ] )
    then
        warn "Invalid package target '${PACKAGE_TARGET}', defaulting to ${DEFAULT_PACKAGE_TARGET}"
        PACKAGE_TARGET="${DEFAULT_PACKAGE_TARGET}"
    fi

    PACKAGE_TARGET="${PACKAGE_TARGET:-}"
    cargo new --name "${PACKAGE_NAME}" --vcs none --${PACKAGE_TARGET} "src/${PACKAGE_DIRECTORY}"

    # Update index document for documentation to point to package
    sed -i'' \
        -e "s/template_repo_rs/${PACKAGE_NAME}/g" \
        docs/index.html
}

run-kill-postgres() {
    info "Stopping Postgres server"
    docker ps | grep postgres | awk '{print $1}' | xargs docker kill 2>/dev/null
}

run-lint() {
    info "Linting code with Clippy"
    cargo clippy "${@}"
}

run-make-docs() {
    info "Compiling package documentation"
    local DOC_BUILD_DIR=$(mktemp -d)
    cargo doc --no-deps --target-dir "${DOC_BUILD_DIR}" "${@}"
    mv ${DOC_BUILD_DIR}/doc/* docs/
    rm -rf "${DOC_BUILD_DIR}"
}

run-publish() {
    info "Creating and publishing distribution packages to crates.io"
    cargo publish "${@}"
}

run-run-cp() {
    info "Running control plane server"
    cargo run -p hare-control-plane-server "${@}" 0.0.0.0:5001
}

run-run-postgres() {
    info "Running Postgres server"
    docker run \
        --rm \
        -d \
        -e POSTGRES_DB=hare \
        -e POSTGRES_PASSWORD=postgres \
        postgres:17-alpine
}

run-run-web() {
    info "Running web server"
    cargo run -p hare-web-server "${@}" 0.0.0.0:8001
}

run-shell() {
    info "Entering shell"
    bash
}

run-test-dp-postgres() {
    local POSTGRES_SERVER_IP=$(docker ps | grep postgres | awk '{print $1}' | xargs docker inspect | grep '"IPAddress"' | head -n1 | grep -Eo '[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}' | tr -d '[[:space:]]')
    if [ -z $POSTGRES_SERVER_IP ]
    then
        error "Postgres server container isn't running"
        exit 1
    fi

    info "Running data plane tests for Postgres"
    export DATABASE_URL="postgres://postgres:postgres@${POSTGRES_SERVER_IP}/hare"
    cargo test -p hare-data-plane-client --no-default-features --features postgres
}

run-test-dp-sqlite() {
    info "Running data plane tests for SQLite"
    export DATABASE_URL="sqlite:///project/src/hare-data-plane-client/hare.db"
    cargo test -p hare-data-plane-client --no-default-features --features sqlite
}

run-test() {
    local TEST_ARGS="$(remove-profile-flags ${@})"
    export CARGO_INCREMENTAL=0 
    export RUSTFLAGS="-Cinstrument-coverage"

    local TARGET_PLATFORM="$(echo ${@} | grep -o '\-\-target [^ ]\+' | sed 's/--target//g' | tr -d '[:space:]')"
    if [ -z "${TARGET_PLATFORM}" ]
    then
        TARGET_ROOT_DIRECTORY="./target/debug"
    else
        TARGET_ROOT_DIRECTORY="./target/${TARGET_PLATFORM}"
    fi
    export LLVM_PROFILE_FILE="${TARGET_ROOT_DIRECTORY}/coverage/hare-%p-%m.profraw"

    # See https://github.com/mozilla/grcov?tab=readme-ov-file#example-how-to-generate-source-based-coverage-for-a-rust-project
    # for documentation on generating source-based coverage for a Rust project
    info "Compiling package with coverage information"
    cargo build ${TEST_ARGS}
    
    run-test-dp-sqlite
    run-test-dp-postgres

    info "Running non-data plane tests"
    cargo test --workspace --exclude hare-data-plane-client "${@}"

    # TODO: Fix coverage report to reflect actual test coverage
    # info "Generating coverage report with grcov"
    # grcov "${TARGET_ROOT_DIRECTORY}/coverage/" -s . --binary-path "${TARGET_ROOT_DIRECTORY}/" -t html --branch --ignore-not-existing -o "${TARGET_ROOT_DIRECTORY}/coverage/"
    # rm -rf ${TARGET_ROOT_DIRECTORY}/coverage/*.profraw

    unset CARGO_INCREMENTAL
    unset LLVM_PROFILE_FILE
    unset RUSTC_BOOTSTRAP
    unset RUSTDOCFLAGS
    unset RUSTFLAGS
}

run-update-deps() {
    info "Updating dependencies"
    cargo update "${@}"
}


################
##### Main #####
################

print-usage() {
    echo "usage: $(basename ${0}) [-h] [SUBCOMMAND]"
    echo
    echo "subcommands:"
    echo "build             cross-build: compile package (default subcommand)"
    echo "build-base        build the build container image"
    echo "build-release     build release containers for Hare components"
    echo "check             cross-check: check package for errors"
    echo "check-deps        cargo-deny: check dependencies for license compliance, security notices, and trusted sources"
    echo "clean             cargo-clean: remove Cargo build artifacts"
    echo "exec              execute arbitrary shell commands"
    echo "fmt               format code with Rustfmt"
    echo "init              initialize repository (should only be run once)"
    echo "kill-postgres     kill Postgres container"
    echo "lint              lint code with Clippy"
    echo "make-docs         cargo-doc: compile package documentation"
    echo "publish           publish package to crates.io"
    echo "push-base         push build container image to registry"
    echo "run-cp            run the control plane server on localhost"
    echo "run-postgres      run Postgres server on localhost"
    echo "run-web           run the web server on localhost"
    echo "shell             start Bash shell"
    echo "test              cross-test: run unit, documentation, and integration tests and code coverage"
    echo "test-dp-postgres  run data plane tests for the Postgres backend"
    echo "test-dp-sqlite    run data plane tests for the SQLite backend"
    echo "update-deps       cargo-update: update dependencies in Cargo.lock file"
    echo
    echo "optional arguments:"
    echo "-h, --help        show this help message and exit"
    echo "-l, --local       run command on host system instead of build container"
    echo "-c, --container   run command in build container"
    echo
}


while :
do
    case "${1:-}" in
        -c|--container)
            shift
            RUNTIME_CONTEXT="container"
        ;;
        -h|--help)
            print-usage
            exit 0
        ;;
        -l|--local)
            shift
            RUNTIME_CONTEXT="local"
        ;;
        *)
            break
        ;;
    esac
done

if [ -z "${1:-}" ]
then
    COMMAND="${DEFAULT_COMMAND}"
else
    COMMAND="${1}"
    shift
fi

# These commands should explicitly run locally
if ( \
    [ "${COMMAND}" = "build-base" ] \
    || [ "${COMMAND}" = "build-release" ] \
    || [ "${COMMAND}" = "push-base" ] \
    || [ "${COMMAND}" = "run-postgres" ] \
    || [ "${COMMAND}" = "kill-postgres" ]
)
then
    RUNTIME_CONTEXT="local"
fi

run-command "${COMMAND}" "${@}"
