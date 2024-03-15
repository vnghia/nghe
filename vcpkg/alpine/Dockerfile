FROM rust:1.76.0-alpine3.19

ARG VCPKG_BINARY_SOURCES="clear;x-gha,readwrite"
ARG ACTIONS_CACHE_URL
ENV ACTIONS_CACHE_URL ${ACTIONS_CACHE_URL}
ARG ACTIONS_RUNTIME_TOKEN
ENV ACTIONS_RUNTIME_TOKEN ${ACTIONS_RUNTIME_TOKEN}

RUN apk add --update --no-cache build-base

ENV VCPKG_FORCE_SYSTEM_BINARIES=1
RUN cargo install cargo-vcpkg

RUN apk add --update --no-cache bash python3 \
  pkgconf cmake ninja-build zip unzip curl git \
  nasm flex bison musl-dev linux-headers musl-libintl mold
RUN ln -s /usr/lib/ninja-build/bin/ninja /usr/bin/ninja

WORKDIR /build
COPY vcpkg/Cargo.toml ./Cargo.toml
COPY vcpkg/src/main.rs ./src/main.rs
COPY vcpkg/triplets/ ./vcpkg/triplets/
COPY vcpkg/alpine/build.sh ./
RUN ./build.sh
