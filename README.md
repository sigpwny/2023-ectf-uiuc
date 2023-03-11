# eCTF 2023 (UIUC)

This repository contains SIGPwny's (University of Illinois Urbana-Champaign) Rust implementation of a car and fob system, PwnyPARED. It includes the following features:

- Unlocking a car with a paired fob
- Pairing an unpaired fob with a paired fob
- Enabling up to 3 features on a car through a provided fob

PwnyPARED was developed to be as secure as possible. Rust was used for its memory safety and the underlying protocol was designed to take full advantage of asymmetric signing using elliptic curve cryptography (P256). Since the TM4C123GXL board is not equipped with a hardware random number generator, we also developed our own RNG based on true random sources, including the state of SRAM, the internal CPU temperature, and the hardware timer value based on user input.

## Documentation

Our code is well-commented and should be easy to follow. We have also documented other important information below:

- [PwnyPARED protocol](./docs/protocol.md)
- [States and variables](./docs/state.md)

## Building and Flashing

This repository is designed to be compatible with MITRE's [eCTF tooling](https://github.com/mitre-cyber-academy/2023-ectf-tools). Please follow the steps there to build and flash firmware.

## Developing

If you want, you can quickly run the code on the boards in a tethered manner. Note that this re-flashes the board and EEPROM will be reset. You will need to manually write values to EEPROM if you would like to test EEPROM reading.

```
cargo run --bin <fob||car||sigpwny-ectf-2023>
```

Before you can run the above command, you will need to run OpenOCD in a separate process since GDB will start and attempt to connect to it. You will also need to have Rust Nightly and the `arm-none-eabi` toolchain installed. Please reference the deployment Dockerfile for more information.

### Logging
Log messages can be printed using our `log!()` macro. These are not added in release mode. Note that using the log macro can affect timing in certain cases and disrupt message transactions, so exercise caution when using them.
