#![no_std]

pub mod tiva;
pub mod driverlib;

use core::{slice, array::from_fn};

use driverlib::{get_temp_samples, get_tick_timer};
use p256_cortex_m4::{SecretKey, Signature, PublicKey};
use rand_chacha::rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};
pub use tiva::board::Board;

/// Sets up the Tiva development board. This includes setting up all the
/// peripherals we use for eCTF, including EEPROM, UART, and GPIO.
/// See wrapper.c for more information.
pub fn setup_board() -> Board {
    let board = Board::new();
    driverlib::init_system();
    board
}

/// Converts an array of u8 to an array of u32
pub fn bytes_to_words(bytes: &[u8], words: &mut [u32]) {
    assert!(bytes.len() %4 == 0 && words.len() *4 == bytes.len());
    if bytes.len() % 4 == 0 && words.len() *4 == bytes.len() {
        for i in 0..words.len() {
            words[i] = u32::from_ne_bytes(bytes[i * 4..(i + 1) *  4].try_into().unwrap());
        }
    }
}

/// Converts an array of u32 to an array of u8
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

/// Reads all of SRAM and hashes it to get a 32-byte entropy value.
pub fn get_ram_entropy() -> [u8; 32] {
    let memory: &[u8];
    unsafe {
        // maybe is secure?
        memory = slice::from_raw_parts(0x20000000 as *const u8, 0x00008000);
    }
    sha256(memory)
}

/// Gets 1024 samples from the temperature sensor and hashes them to get a
/// 32-byte entropy value.
pub fn get_temp_entropy() -> [u8; 32] {
    let mut samples = [0u32; 8];
    let mut samples_lsb;
    let mut hash = Sha256::new();
    for _ in 0..1024 {
        get_temp_samples(&mut samples);
        samples_lsb = samples.map(|x| x as u8);
        hash.update(samples_lsb);
    }
    hash.finalize().into()
}

/// Gets 128 samples from the tick timer and hashes them to get a 32-byte
/// entropy value.
pub fn get_timer_entropy() -> [u8; 32] {
    let mut hash = Sha256::new();
    for _ in 0..128 {
        hash.update(get_tick_timer().to_ne_bytes())
    }
    hash.finalize().into()
}

/// Combines the entropy from the RAM, temperature sensor, and tick timer to
/// get a 32-byte entropy value.
pub fn get_combined_entropy() -> [u8; 32] {
    let ram_entropy = get_ram_entropy();
    let temp_entropy = get_temp_entropy();
    let timer_entropy = get_timer_entropy();
    from_fn(|i| ram_entropy[i] ^ temp_entropy[i] ^ timer_entropy[i])
}

/// Hashes a message using SHA-256.
/// https://github.com/ycrypto/p256-cortex-m4/blob/290b275c08ef8964eda308ea56c888c1cf0fa06a/src/lib.rs#L27-L33
pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(message);
    let data = hash.finalize();
    data.into()
}

/// Signs a message using the ECDSA algorithm.
pub trait Signer {
    fn sign(&self, message: &[u8], rng: impl CryptoRng + RngCore) -> Signature;
}

/// Implementation of signature generation using the ECDSA algorithm.
impl Signer for SecretKey {
    // https://github.com/ycrypto/p256-cortex-m4/blob/290b275c08ef8964eda308ea56c888c1cf0fa06a/src/cortex_m4.rs#L187-L190
    fn sign(&self, message: &[u8], rng: impl CryptoRng + RngCore) -> Signature {
        let prehashed_message = sha256(message);
        self.sign_prehashed(prehashed_message.as_ref(), rng)
    }
}

/// Verifies a signature using the ECDSA algorithm.
pub trait Verifier {
    fn verify(&self, message: &[u8], signature: &Signature) -> bool;
}

/// Implementation of signature validation using the ECDSA algorithm.
impl Verifier for PublicKey {
    // https://github.com/ycrypto/p256-cortex-m4/blob/290b275c08ef8964eda308ea56c888c1cf0fa06a/src/cortex_m4.rs#L302-L305
    fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        let prehashed_message = sha256(message);
        self.verify_prehashed(prehashed_message.as_ref(), signature)
    }
}
