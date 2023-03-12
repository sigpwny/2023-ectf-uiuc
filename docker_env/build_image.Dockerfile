FROM ubuntu:22.04

RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    make \
    python3.9 \
    python3-pip \
    clang \
    binutils-arm-none-eabi \
    gcc-arm-none-eabi \
    libgmp3-dev \
    curl \
    dos2unix

# Set up Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default nightly
RUN rustup target add thumbv7em-none-eabihf

RUN python3 -m pip install fastecdsa

# Create a revision file to force a rebuild of the image if necessary
RUN echo "1.0.0" > /.revision

WORKDIR /sigpwny

COPY Cargo.toml Cargo.lock memory.x ./
COPY .cargo ./.cargo
RUN mkdir src src/bin && echo "#![no_std] #![no_main] use cortex_m_rt::entry; use core::panic::PanicInfo; #[entry] fn main() -> ! {loop{}} #[panic_handler] fn panic(_: &PanicInfo) -> ! {loop{}}" > src/main.rs && echo "#![no_std]" > src/lib.rs && cp src/main.rs src/bin/car.rs && cp src/main.rs src/bin/fob.rs
RUN cargo build --release --bin sigpwny-ectf-2023

# copy the entire build directory (src, Cargo.toml, etc.) to /sigpwny
COPY . .
