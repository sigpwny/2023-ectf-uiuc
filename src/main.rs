#![no_std]
#![no_main]

use cortex_m_semihosting::{hprintln};
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;
#[allow(unused_imports)]
use tm4c123x_hal;

#[entry]
fn main() -> ! {
    unsafe {
        InitSystem();
    }
    hprintln!("It works!!!").unwrap();
    loop {
        asm::wfi();
    }
}

#[link(name = "drivermini")]
extern "C" {
    fn InitSystem();
}
