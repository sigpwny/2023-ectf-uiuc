#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

use tiva::{
    driverlib::{self, eeprom_read, eeprom_write, start_delay_timer_us, sleep_us, wait_delay_timer, get_tick_timer, get_temp_samples, get_remaining_us_delay_timer},
    log, setup_board, sha256, Board, Signer, Verifier, get_combined_entropy
};

/// This code is not utilized by the final device code. It is used as a test
/// playgroud.

#[entry]
fn main() -> ! {
    let mut board: Board = setup_board();

    log!("Hello, world!");
    write_str_to_host("Hello, world!\n");

    crypto_example();

    eeprom_example();

    entropy_example();

    timer_example();

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

        sleep_us(500_000);
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
    write_str_to_host("Begin gathering entropy\n");
    let entropy = get_combined_entropy();
    write_str_to_host("entropy: ");
    write_to_hex(&entropy);
    write_str_to_host("\n");
    write_str_to_host("Temp example samples:\n");
    let mut samples = [0u32; 8];
    for _ in 0..10 {
        get_temp_samples(&mut samples);
        write_str_to_host("Samples: ");
        for s in samples {
            write_to_hex(&s.to_be_bytes());
            write_str_to_host(" ")
        }
        write_str_to_host("\n");
    }
    log!("Entropy test completed");
}

fn timer_example() {
    write_str_to_host("Starting timer example\n");
    // first few should take 1s because the sleep time is less than delay timer
    for i in 0..20 {
        start_delay_timer_us(1_000_000);
        sleep_us((i + 1) * 100_000);
        write_str_to_host("time remaining: ");
        write_to_hex(&get_remaining_us_delay_timer().to_be_bytes());
        write_str_to_host("\n");
        wait_delay_timer();
        write_str_to_host("delay fired at ticker: ");
        write_to_hex(&get_tick_timer().to_be_bytes());
        write_str_to_host("\n");
    }
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
