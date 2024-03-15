FROM rust:1.76.0-alpine3.19

ARG ACTIONS_CACHE_URL
ARG ACTIONS_RUNTIME_TOKEN

RUN apk add --update --no-cache build-base

ENV VCPKG_FORCE_SYSTEM_BINARIES=1
RUN cargo install cargo-vcpkg

RUN apk add --update --no-cache bash python3 \
  pkgconf cmake samurai zip unzip curl git

WORKDIR /build
COPY vcpkg/Cargo.docker.toml ./Cargo.toml
COPY vcpkg/src/main.rs ./src/main.rs
COPY vcpkg/triplets/ ./vcpkg/triplets/
COPY vcpkg/alpine/build.sh ./
RUN ./build.sh

RUN apk add --update --no-cache bash python3 \
  nasm flex bison libintl
RUN git config --global --add safe.directory '*'

COPY vcpkg/Cargo.toml ./Cargo.toml
RUN ./build.sh
