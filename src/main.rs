#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

use tiva::{
    driverlib::{self, eeprom_read, eeprom_write},
    log, setup_board, sha256, Board, Signer, Verifier, get_combined_entropy
};

#[entry]
fn main() -> ! {
    let mut board: Board = setup_board();

    log!("Hello, world!");
    write_str_to_host("Hello, world!\n");

    crypto_example();

    eeprom_example();

    entropy_example();

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

    let buf: [u8; 64] = signature.to_untagged_bytes();
    write_str_to_host("Signature: ");
    write_to_hex(&buf);
    write_str_to_host("\n");

    let verifying_key = signing_key.public_key();
    assert!(verifying_key.verify(message, &signature));
    log!("Signature verified!");

    // hashing example
    let result = sha256(&b"hello world"[..]);

    write_str_to_host("Hash: ");
    write_to_hex(&result);
    write_str_to_host("\n");
}

fn led_and_uart_example(board: &mut Board) -> ! {
    let mut toggle = true;
    loop {
        if driverlib::read_sw_1() {
            log!("SW1 is pressed");
            driverlib::uart_writeb_host(b'!');
        } else {
            log!("SW1 is not pressed");
            driverlib::uart_writeb_host('_' as u8);
        }

        if toggle {
            board.led_green.set_high().unwrap();
        } else {
            board.led_green.set_low().unwrap();
        }
        toggle = !toggle;

        wait(2_000_000);
    }
}

fn wait(length: u32) {
    for _ in 0..length {
        cortex_m::asm::nop();
    }
}

fn eeprom_example() {
    const WRITE_LOC: u32 = 0;
    const WRITE_SIZE: usize = 512;
    // initalize our data
    let mut wdata: [u32; WRITE_SIZE] = [0; WRITE_SIZE];
    for address in 0..WRITE_SIZE {
        wdata[address] = address as u32;
    }

    // Write Our data
    eeprom_write(&wdata, WRITE_LOC);

    // Read out data
    let mut rdata: [u32; WRITE_SIZE] = [0; WRITE_SIZE];
    eeprom_read(&mut rdata, WRITE_LOC);
    for address in 0..WRITE_SIZE {
        assert!(wdata[address] == rdata[address]);
    }

    log!("EEPROM Tests passed");
}

fn entropy_example() {
    write_str_to_host("Begin gathering entropy");
    let entropy = get_combined_entropy();
    write_str_to_host("entropy: ");
    write_to_hex(&entropy);
    write_str_to_host("\n");
    log!("Entropy test completed");
}

fn write_str_to_host(s: &str) {
    for c in s.bytes() {
        driverlib::uart_writeb_host(c);
    }
}

fn write_to_hex(data: &[u8]) {
    for byte in data {
        let hex1: u8 = byte_to_half_hex(byte >> 4);
        let hex2: u8 = byte_to_half_hex(byte & 0x0F);
        driverlib::uart_writeb_host(hex1 as u8);
        driverlib::uart_writeb_host(hex2 as u8);
    }
}

fn byte_to_half_hex(byte: u8) -> u8 {
    if byte > 9 {
        byte + 87
    } else {
        byte + 48
    }
}
