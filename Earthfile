VERSION --use-function-keyword --arg-scope-and-set 0.7

FROM alpine:3.19

source:
    WORKDIR source-files
    COPY Cargo.toml Cargo.lock README.md .
    COPY --dir \
        redact-composer \
        redact-composer-core \
        redact-composer-derive \
        redact-composer-midi \
        redact-composer-musical \
        .

    SAVE ARTIFACT *

# ci-checks Runs the same checks run by CI. Useful for local testing before submitting a PR.
ci-checks:
    BUILD +rust-fmt
    BUILD +clippy
    BUILD +check-typos
    BUILD +docs
    BUILD +build-examples
    BUILD +test-matrix

# test Runs `cargo test` on a specific toolchain (default:stable).
test:
    ARG toolchain
    FROM +rust-img --toolchain=${toolchain:-stable}
    COPY +source/ .

    RUN cargo test

# test-matrix Runs `cargo test` on a list of toolchains (default:stable,nightly,1.70.0).
test-matrix:
    ARG toolchains

    FOR --sep="," toolchain IN ${toolchains:-stable,nightly,1.70.0}
        BUILD +test --toolchain=$toolchain
    END

# rust-fmt Checks that `cargo fmt` has been applied.
rust-fmt:
    DO +PREPARE_WORKSPACE --components=rustfmt

    RUN cargo fmt --all -- --check

# clippy Runs `cargo clippy --all-features --tests -- -Dclippy::all`
clippy:
    DO +PREPARE_WORKSPACE --components=clippy

    RUN cargo clippy --all-features --tests -- -Dclippy::all

# check-typos Checks for typos in source code/documentation.
check-typos:
    DO +PREPARE_WORKSPACE
    RUN cargo install typos-cli

    RUN typos

# docs Creates the rust docs for the project.
docs:
    DO +PREPARE_WORKSPACE

    RUN cargo doc --all-features --no-deps

    SAVE ARTIFACT ./target/doc/* docs/

# coverage Runs tests with coverage via llvm-cov.
coverage:
    DO +PREPARE_WORKSPACE --toolchain=nightly --components=llvm-tools
    RUN cargo +nightly install cargo-llvm-cov

    RUN cargo +nightly llvm-cov --all-features --workspace --doctests --html --output-dir ./llvm-cov/html/
    RUN cargo +nightly llvm-cov report --lcov --output-path ./llvm-cov/lcov.info
    RUN cargo +nightly llvm-cov report --cobertura --output-path ./llvm-cov/lcov.xml
    RUN cargo +nightly llvm-cov report --json --output-path ./llvm-cov/lcov.json

    SAVE ARTIFACT ./llvm-cov/* coverage/

# build-examples Builds the examples for redact-composer
build-examples:
    DO +PREPARE_WORKSPACE

    RUN cargo build --release --package redact-composer --example simple

PREPARE_WORKSPACE:
    FUNCTION
    ARG EARTHLY_TARGET_NAME
    ARG toolchain
    ARG components

    FROM +rust-img --toolchain=${toolchain:-stable} --components=$components
    WORKDIR $EARTHLY_TARGET_NAME
    COPY +source/ .

rust-img:
    ARG toolchain
    ARG components

    FROM +rust-base --toolchain=${toolchain:-stable}

    FOR --sep=" ," component IN $components
        RUN rustup component add $component
    END

rust-base:
    ARG toolchain

    IF [ $toolchain = "stable" ]
        FROM rust
        RUN rustup default stable
    ELSE IF [ $toolchain = "nightly" ]
        FROM rustlang/rust:nightly
    ELSE
        FROM rust:${toolchain}
    END
