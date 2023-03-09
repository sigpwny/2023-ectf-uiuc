FROM ubuntu:22.04

RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    make \
    python3.9 \
    python3-pip \
    clang \
    binutils-arm-none-eabi \
    gcc-arm-none-eabi \
    libgmp3-dev \
    curl

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default nightly

RUN rustup target add thumbv7em-none-eabihf

RUN python3 -m pip install fastecdsa

# copy the entire build directory (src, Cargo.toml, etc.) to /sigpwny
COPY . /sigpwny
