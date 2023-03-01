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
pub fn bytes_to_words(bytes: &[u8], words: &mut [u32]) {
    assert!(bytes.len() %4 == 0 && words.len() *4 == bytes.len());
    if bytes.len() % 4 == 0 && words.len() *4 == bytes.len() {
        for i in 0..words.len() {
            words[i] = u32::from_ne_bytes(bytes[i * 4..(i + 1) *  4].try_into().unwrap());
        }
    }
}

// convert an array of u32 to an array of u8
pub fn words_to_bytes(words: &[u32], bytes: &mut [u8]) {
    assert!(bytes.len() %4 == 0 && words.len() *4 == bytes.len());
    if bytes.len() % 4 == 0 && words.len() *4 == bytes.len() {
        for i in 0..words.len() {
            let word_bytes = words[i].to_ne_bytes();
            bytes[i * 4] = word_bytes[0];
            bytes[i * 4 + 1] = word_bytes[1];
            bytes[i * 4 + 2] = word_bytes[2];
            bytes[i * 4 + 3] = word_bytes[3];
        }
    }
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
