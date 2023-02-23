#![no_std]
#![no_main]

// use panic_halt as _;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

use tiva::{
    driverlib::{self/*, eeprom_read, eeprom_write*/},
    log, setup_board, Board,
};

// define magic bytes for protocol
const MAGIC_PAIR_REQ: u8 = 0x40; const LEN_PAIR_REQ: u8 = 4;
const OFFSET_PAIR_REQ__PIN: u8 = 1; const LEN_PAIR_REQ__PIN: u8 = 3;
const MAGIC_PAIR_SYN: u8 = 0x41; const LEN_PAIR_SYN: u8 = 4;
const OFFSET_PAIR_SYN__PIN: u8 = 1; const LEN_PAIR_SYN__PIN: u8 = 3;
const MAGIC_PAIR_ACK: u8 = 0x42; const LEN_PAIR_ACK: u8 = 1;
const MAGIC_PAIR_FIN: u8 = 0x43; const LEN_PAIR_FIN: u8 = 1; // this length should eventually include the transferred secret

// FOR TESTING ONLY
const LEN_CAR_SECRET: u8 = 32; // 32 bytes
const LEN_FEATURE: u8 = 32; // 32 bytes
const FOB_IS_PAIRED: bool = true;

#[entry]
fn main() -> ! {
    let mut board: Board = setup_board();

    log!("This is fob!");

    board.led_blue.set_high().unwrap(); // Turn on blue LED

    // Write "supersecret!" to EEPROM
    // Store "supersecret!" as a u32 array
    let secret_1: u32 = 0x73757065;
    let secret_2: u32 = 0x72736563;
    let secret_3: u32 = 0x72657421;
    let secret = [secret_1, secret_2, secret_3];
    // Write to EEPROM
    driverlib::eeprom_write(&secret, 0);
    // Read from EEPROM
    let mut read_out: [u32; 3] = [0; 3];
    driverlib::eeprom_read(&mut read_out, 0);
    log!("EEPROM readout: {:x?}", read_out);

    // Define string of characters to write to UART
    let string: &[u8] = "This is fob, but over host serial!\n".as_bytes();
    for i in 0..string.len() {
        driverlib::uart_writeb_host(string[i]);
    }

    // Read from host UART, log output, and write back to host UART
    loop {
        // if driverlib::uart_avail_host() {
        // driverlib::uart_writeb_host(data);
        let data: u8 = driverlib::uart_readb_host();
        match (data) {
            MAGIC_PAIR_REQ => {
                if (FOB_IS_PAIRED) {
                    paired_fob_pairing();
                }
            }
            MAGIC_PAIR_SYN => {
                if (!FOB_IS_PAIRED) {
                    unpaired_fob_pairing();
                }
            }
            _ => {
                // probably timeout? or continue listening
            }
        }
    }
    // loop{}
}


// Use driverlib::uart_readb_host() to read a byte from the host UART
// Use driverlib::uart_writeb_host(data); to write a byte to the host UART
// Use driverlib::uart_readb_board() to read a byte from the board UART
// Use driverlib::uart_writeb_board(data); to write a byte to the board UART

fn paired_fob_pairing() -> ! {
    // we just received a PAIR_REQ
}

fn unpaired_fob_pairing() -> ! {
    // we just received a PAIR_SYN, so now we need to read the PIN
}