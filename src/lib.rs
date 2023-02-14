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
