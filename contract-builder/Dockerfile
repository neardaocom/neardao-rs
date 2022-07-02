FROM rust:1.60.0

LABEL description="Container for builds"

RUN rustup default 1.60.0
RUN rustup target add wasm32-unknown-unknown

RUN apt-get update && apt-get install -y \
    git \
    less \
    vim \
    llvm \
    clang \
    && rm -rf /var/lib/apt/lists/*  

RUN wget https://github.com/WebAssembly/binaryen/releases/download/version_109/binaryen-version_109-x86_64-linux.tar.gz
RUN tar -xf binaryen-version_109-x86_64-linux.tar.gz
COPY /binaryen-version_109/bin/* usr/local/bin
RUN rm -rf /binaryen-version_109 && rm /binaryen-version_109-x86_64-linux.tar.gz