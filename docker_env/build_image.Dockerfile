FROM ubuntu:22.04

RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    make \
    python3.9 \
    clang \
    binutils-arm-none-eabi \
    gcc-arm-none-eabi \
    curl

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default nightly

RUN rustup target add thumbv7em-none-eabi