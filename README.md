# Rust Branch How To
## Dependencies
You do not need UNIFLASH, the device drivers (I think?), or GCC/GDB <ins>for your host machine</ins> (yes, this works on M1 Mac).

### GNU Arm Toolchain
You can download the `arm-none-eabi` tools and libraries [here](https://developer.arm.com/Tools%20and%20Software/GNU%20Toolchain), but they install in an annoying format. I would recommend using your package manager instead to avoid PATH difficulties.

**macOS**
```
$ brew install armmbed/formulae/arm-none-eabi-gcc
```

**Windows**

Use the [Windows 32-bit Installer](https://developer.arm.com/-/media/Files/downloads/gnu-rm/10.3-2021.10/gcc-arm-none-eabi-10.3-2021.10-win32.exe?rev=29bb46cfa0434fbda93abb33c1d480e6&hash=B2C5AAE07841929A0D0BF460896D6E52). 

*Note: Make sure to check the `Add path to environment variable` option before you click the Finish button for the installation.*

### OpenOCD
Use your package manager ¯\\\_\(ツ\)\_/¯

**macOS**
```
$ brew install openocd
```

**Windows**

Use [MSYS2](http://msys2.org/)(make sure it is up to date):
```
pacman -S openocd
```

## Setup

### Getting Rust
**macOS**
```
# install Rust using rustup
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# switch to nightly
$ rustup default nightly
# add Cortex-M target
$ rustup target add thumbv7em-none-eabihf
```
Double check you're all good:
```
$ rustc -V
rustc 1.69.0-nightly (50d3ba5bc 2023-02-04)
```
Remember that you can run `rustup default stable` to switch back to stable if you do other Rust development on your machine.

**Windows**

Run the [official installer](https://www.rust-lang.org/tools/install) and follow the instructions. (you can do that right?)

Double check you're all good:
```
rustc -V
rustc 1.69.0-nightly (50d3ba5bc 2023-02-04)
```
Remember that you can run `rustup default stable` to switch back to stable if you do other Rust development on your machine.

### Does it work?
```
$ cargo build --bin sigpwny-ectf-2023
```
*Hopefully* you don't get errors. If you do, ~~cry~~ ping @Shorden on Discord and I will try to help.

If that was successful, run
```
$ openocd
Info : ICDI Firmware version: 9270
Info : [tm4c123gh6pm.cpu] Cortex-M4 r0p1 processor detected
Info : [tm4c123gh6pm.cpu] target has 6 breakpoints, 4 watchpoints
Info : starting gdb server for tm4c123gh6pm.cpu on 3333
Info : Listening on port 3333 for gdb connections
```
Then, in **a different terminal**,
```
$ cargo run
...
Reading symbols from target/thumbv7em-none-eabihf/debug/car...
0x00000390 in core::fmt::Arguments::new_v1 ()
Breakpoint 1 at 0x356
Breakpoint 2 at 0x11a4: file src/lib.rs, line 560.
Breakpoint 3 at 0x530
Breakpoint 4 at 0x2c0: file src/main.rs, line 15.
semihosting is enabled
Loading section .vector_table, size 0x26c lma 0x0
Loading section .text, size 0xf44 lma 0x26c
Loading section .rodata, size 0x408 lma 0x11b0
Start address 0x26c, load size 5560
Transfer rate: 5 KB/sec, 1853 bytes/write.
Note: automatically using hardware breakpoints for read-only addresses.
halted: PC: 0x0000026e
0x0000026e in Reset ()
(gdb)
```
Make sure you check that `semihosting is enabled` and the program was loaded onto the board properly (this is very easy to screw up if you don't use `openocd.gdb`). You can send commands to OpenOCD in GDB with `monitor <OpenOCD command>`. From here, run the program and switch back to your OpenOCD terminal:
```
[tm4c123gh6pm.cpu] halted due to debug-request, current mode: Thread
xPSR: 0x01000000 pc: 0x0000026c msp: 0x20008000, semihosting
Info : halted: PC: 0x0000026e
Error: memory read failed: 0x7
Info : halted: PC: 0x000002c6
It works!!!
```
Now start helping me write an abstraction layer!

And remember, `unsafe` is banned :)

## Errata
Please PR or DM me so I can add to this list as things come up.
### Compiling
- Make sure you include the `use tm4c123x_hal` statement in `main.rs` EVEN IF you don't use anything from the crate.
- (Tangential) If you want to create a project from the template [here](https://github.com/rust-embedded/cortex-m-quickstart), make sure you use `cargo generate` instead of just cloning the repo.
### OpenOCD
- `Error: SRST error`: This is believed to be a problem with OpenOCD. This should not cause any problems.
- `Error: Can't find openocd.cfg`: Make sure you're in the project root when you run `openocd`. Also... please use the config.
### GDB
- Make sure if you use `gdb-multiarch` instead of `arm-none-eabi-gdb` you change the corresponding line in `.cargo/config.toml`.

## Resources
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [The Embedded Rust Book](https://doc.rust-lang.org/beta/embedded-book/)

# TODO
- Add Cortex-Debug functionality for VS Code
- Migrate secrets/gen scripts and whatnot from `car`, `fob` directories
- I'd rather use cargo with `build.rs` instead of Makefiles, but if we must then setup make
- Make sure Docker works with our stuff
