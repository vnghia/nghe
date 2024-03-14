FROM rust:1.76.0

ARG ACTIONS_CACHE_URL
ARG ACTIONS_RUNTIME_TOKEN

RUN apt-get update && apt-get --assume-yes --no-install-recommends install \
  build-essential && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-vcpkg

RUN apt-get update && apt-get --assume-yes --no-install-recommends install \
  cmake ninja-build curl zip unzip tar \
  nasm

WORKDIR /build
COPY vcpkg/Cargo.toml ./Cargo.toml
COPY vcpkg/src/main.rs ./src/main.rs
COPY vcpkg/triplets/ ./vcpkg/triplets/
RUN cargo vcpkg --verbose build --target $(uname -m)-unknown-linux-gnu
