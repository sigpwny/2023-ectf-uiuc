# eCTF 2023 (UIUC)

This repository contains SIGPwny's (University of Illinois Urbana-Champaign) Rust implementation of a car and fob system, PwnyPARED. It includes the following features:

- Unlocking a car with a paired fob
- Pairing a new fob with an already paired fob
- Enabling up to 3 features on a fob to start the car

PwnyPARED was developed to be as secure as possible. It features the following security features:

- Rust was used for memory safety. Panics are denoted by a flashing red LED.
- The underlying protocol was designed to take full advantage of asymmetric signing using elliptic curve cryptography (P256).
- Since the TM4C123GXL board is not equipped with a hardware random number generator, we developed our own RNG which draws entropy from volatile sources, including all of SRAM, the internal CPU temperature, and hardware timer values at certain user-initiated events.

## Documentation

Our code is well-commented and should be easy to follow. Fob code can be found in [fob.rs](./docker_env/src/bin/fob.rs) and car code can be found in [car.rs](./docker_env/src/bin/car.rs). Helper functions are defined in [lib.rs](./docker_env/src/lib.rs). We also use the Tiva driverlib library for some tasks using Rust to C bindings, which are defined in [wrapper.c](./docker_env/tivaware/driverlib/wrapper.c) and [driverlib.rs](./docker_env/src/driverlib.rs).

Other useful information is documented below:

- [Design Document](./docs/design-v1.3.pdf)
- [PwnyPARED protocol](./docs/protocol.md)
- [States and variables](./docs/state.md)

## Building and Flashing

This repository is designed to be compatible with MITRE's [eCTF tooling](https://github.com/mitre-cyber-academy/2023-ectf-tools). Please follow the steps there to build and flash firmware.

## Developing

If you want, you can quickly run our code on the boards in a tethered manner. Note that this re-flashes the board and EEPROM will be reset. You will need to manually write values to EEPROM if you would like to test EEPROM reading.

```
cargo run --bin <fob||car||sigpwny-ectf-2023>
```

Before you can run the above command, you will need to run OpenOCD in a separate process since GDB will start and attempt to connect to it. You will also need to have Rust Nightly and the `arm-none-eabi` toolchain installed. Please reference the deployment Dockerfile for more information.

### Logging
Log messages can be printed using our `log!()` macro. These are not added in release mode. Note that using the log macro can affect timing and disrupt message transactions  in certain cases, so exercise caution when using them.
