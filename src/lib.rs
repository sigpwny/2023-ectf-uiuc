#![no_std]

pub mod tiva;
pub mod driverlib;

pub use tiva::board::Board;

/// Sets up the Tiva development board. This includes setting up all the
/// peripherals we use for eCTF, including EEPROM, UART, and GPIO.
pub fn setup_board() -> Board {
    let board = Board::new();
    driverlib::init_system();
    board
}

/// Pass directly to hprintln if we are not in debug mode. Otherwise, do
/// nothing.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            use cortex_m_semihosting::hprintln;
            hprintln!($($arg)*).unwrap();
        }
    }
}
