#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::OutputPin;

use tiva::{driverlib, setup_board, Board};

#[entry]
fn main() -> ! {
    let mut board: Board = setup_board();

    hprintln!("Hello, world!").unwrap();

    crypto_example();

    led_and_uart_example(&mut board)
}

fn crypto_example() {
    // key generation
    use rand_chacha::rand_core::SeedableRng;
    // signing
    use k256::ecdsa::{signature::Signer, Signature, SigningKey};
    // verifying
    use k256::ecdsa::{signature::Verifier, VerifyingKey};

    // The only time we need crypto on the device is:
    // 1. The car generates a random nonce, which the fob signs and the car verifies
    // 2. The car verifies that a feature was signed by the factory

    // we could get a source of randomness from the SRAM initial state
    let mut rng = rand_chacha::ChaChaRng::from_seed([0; 32]);

    // keypair generation should be done on the host, but here's how to do it on the device
    let signing_key = SigningKey::random(&mut rng); // Serialize with `::to_bytes()`
    let message: &[u8] = b"Some text";
    let signature: Signature = signing_key.sign(message);
    // hprintln!("Signature: {:?}", signature).unwrap();

    let verifying_key = VerifyingKey::from(&signing_key); // Serialize with `::to_encoded_point()`
    assert!(verifying_key.verify(message, &signature).is_ok());
    hprintln!("Signature verified!").unwrap();
}

fn led_and_uart_example(board: &mut Board) -> ! {
    let mut toggle = true;
    loop {
        if driverlib::check_switch() {
            hprintln!("SW1 is pressed").unwrap();
        } else {
            hprintln!("SW1 is not pressed").unwrap();
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
