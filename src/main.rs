#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

use tiva::{
    driverlib::{self, eeprom_read, eeprom_write},
    log, setup_board, Board,
};

#[entry]
fn main() -> ! {
    let mut board: Board = setup_board();

    log!("Hello, world!");

    crypto_example();

    led_and_uart_example(&mut board)
}

fn crypto_example() {
    // key generation
    use p256_cortex_m4::SecretKey;
    use rand_chacha::rand_core::SeedableRng;

    // The only time we need crypto on the device is:
    // 1. The car generates a random nonce, which the fob signs and the car verifies
    // 2. The car verifies that a feature was signed by the factory

    // we could get a source of randomness from the SRAM initial state
    let mut rng = rand_chacha::ChaChaRng::from_seed([0; 32]);

    // keypair generation should be done on the host, but here's how to do it on the device
    let signing_key = SecretKey::random(&mut rng);
    let message: &[u8] = b"Some text";
    let signature = signing_key.sign(message, &mut rng);

    let verifying_key = signing_key.public_key();
    assert!(verifying_key.verify(message, &signature));
    log!("Signature verified!");

    // hashing example
    use p256_cortex_m4::sha256;

    let result = sha256(&b"hello world"[..]);
    log!("Hash: {:?}", result);
}

fn led_and_uart_example(board: &mut Board) -> ! {
    let mut toggle = true;
    loop {
        if driverlib::check_switch() {
            log!("SW1 is pressed");
        } else {
            log!("SW1 is not pressed");
        }
        driverlib::uart_writeb_host('a' as u8);

        if toggle {
            board.led_green.set_high().unwrap();
        } else {
            board.led_green.set_low().unwrap();
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

fn eeprom() {
    // initalize our data
    let mut wdata: [u32; 64];
    for address in 0..EEPROM_SIZE {
        wdata[address] = address;
    }

    // Write Our data
    eeprom_write(&wdata, UNLOCK_EEPROM_LOC);

    // Read out data
    let mut rdata: [u32; 64];
    eeprom_read(&rdata, UNLOCK_EEPROM_LOC);
    for address in 0..EEPROM_SIZE {
        assert!(wdata[address] == rdata[address]);
    }

    log!("EEPROM Tests passed");
    // #define UNLOCK_EEPROM_LOC 0x7C0
    // #define UNLOCK_EEPROM_SIZE 64
}
