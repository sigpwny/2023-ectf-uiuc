#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use embedded_hal::digital::v2::OutputPin;

use tiva::{driverlib, setup_board, Board};

#[entry]
fn main() -> ! {
    let mut board: Board = setup_board();
    let _ = board; // suppress unused warning

    hprintln!("Hello, world!").unwrap();

    let mut toggle = true;
    loop {
        if driverlib::check_switch() {
            hprintln!("SW1 is pressed").unwrap();
        } else {
            hprintln!("SW1 is not pressed").unwrap();
        }
        driverlib::uart_writeb_host('a' as u8);

        if toggle {
            board.led_red.set_high().unwrap();
        } else {
            board.led_red.set_low().unwrap();
        }
        toggle = !toggle;

        wait(1e5 as u32);
    }
}

fn wait(length: u32) {
    for _ in 0..length {
        cortex_m::asm::nop();
    }
}
