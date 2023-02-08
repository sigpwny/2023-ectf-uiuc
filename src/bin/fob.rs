#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
#[allow(unused_imports)]
use tm4c123x_hal;


#[entry]
fn main() -> ! {
    loop {}
}
