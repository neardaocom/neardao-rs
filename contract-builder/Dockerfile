FROM rust:1.55.0

LABEL description="Container for builds"

RUN rustup default 1.55.0
RUN rustup component add clippy
RUN rustup target add wasm32-unknown-unknown

RUN apt-get update && apt-get install -y \
    git \
    less \
    vim \
    llvm \
    clang \
    && rm -rf /var/lib/apt/lists/*  