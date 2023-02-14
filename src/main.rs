#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use ed25519_dalek::{Keypair, Signer};
use embedded_hal::digital::v2::OutputPin;
use linked_list_allocator::LockedHeap;
use rand_chacha;
use rand_core;
use rand_core::SeedableRng;

use tiva::{driverlib, setup_board, Board};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[entry]
fn main() -> ! {
    // Use the linked list allocator directory instead of embedded-alloc because
    // that interferes with the C tiva driverlib linking (the critical_section
    // crate has some segments which conflict).
    //
    // Ideally, this should be done in a critical section where interrupts are
    // disabled.
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 0x4000;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe {
            ALLOCATOR
                .lock()
                .init(HEAP_MEM.as_ptr() as *mut u8, HEAP_SIZE);
        }
    }

    let mut board: Board = setup_board();

    hprintln!("Hello, world!").unwrap();

    crypto_example();

    led_and_uart_example(&mut board)
}

fn crypto_example() {
    // The only time we need crypto on the device is:
    // 1. The car generates a random nonce, which the fob signs and the car verifies
    // 2. The car verifies that a feature was signed by the factory

    // we could get a source of randomness from the SRAM initial state
    let mut rng = rand_chacha::ChaChaRng::from_seed([0; 32]);

    // keypair generation should be done on the host, but here's how to do it on the device
    let keypair = Keypair::generate(&mut rng);
    let message: &[u8] = b"Some text";
    let signature = keypair.sign(message);
    // hprintln!("Signature: {:?}", signature).unwrap();

    assert!(keypair.verify(message, &signature).is_ok());
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
