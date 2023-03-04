#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

use tiva::{
  driverlib::*,
  driverlib::{self},
  log, setup_board, Board, words_to_bytes, bytes_to_words
};

use p256_cortex_m4::SecretKey;

/**
 * EEPROM state addresses (specifically for car)
 */
const CARMEM_CAR_SECRET:      u32 = 0x100;
const CARMEM_MAN_PUBLIC:      u32 = 0x120;
const CARMEM_FOB_PUBLIC:      u32 = 0x160;
const CARMEM_CAR_ID:          u32 = 0x200;
 
const CARMEM_MSG_FEAT_3:      u32 = 0x700;
const CARMEM_MSG_FEAT_2:      u32 = 0x740;
const CARMEM_MSG_FEAT_1:      u32 = 0x780;
const CARMEM_MSG_UNLOCK:      u32 = 0x7C0;


/**
 * EEPROM state lengths
 */
// in bytes (for sending over UART)
const LEN_FOB_SECRET:         usize = 32;
const LEN_FOB_PUBLIC:         usize = 64;
const LEN_CAR_SECRET:         usize = 32;
const LEN_CAR_PUBLIC:         usize = 64;
const LEN_MAN_SECRET:         usize = 32;
const LEN_MAN_PUBLIC:         usize = 64;
const LEN_CAR_ID:             usize = 4; // 1 byte at heart
const LEN_FEAT:               usize = 4; // 1 byte at heart
const LEN_FEAT_SIG:           usize = 64;

// in words (for accesing EEPROM)
const LENW_FOB_SECRET:        usize = LEN_FOB_SECRET / 4;
const LENW_FOB_PUBLIC:        usize = LEN_FOB_PUBLIC / 4;
const LENW_CAR_SECRET:        usize = LEN_CAR_SECRET / 4;
const LENW_CAR_PUBLIC:        usize = LEN_CAR_PUBLIC / 4;
const LENW_MAN_SECRET:        usize = LEN_MAN_SECRET / 4;
const LENW_MAN_PUBLIC:        usize = LEN_MAN_PUBLIC / 4;
const LENW_CAR_ID:            usize = LEN_CAR_ID / 4;
const LENW_FEAT:              usize = LEN_FEAT / 4;
const LENW_FEAT_SIG:          usize = LEN_FEAT_SIG / 4;

// Unlock specific state
const LEN_NONCE:              usize = 8; // 64-bit nonce
const LEN_NONCE_SIG:          usize = 64;

/**
 * Magic Bytes
 */
// start at 0x60
const MAGIC_UNLOCK_REQ:       u8 = 0x60;
const MAGIC_UNLOCK_CHAL:      u8 = 0x61;
const MAGIC_UNLOCK_RESP:      u8 = 0x62;
const MAGIC_UNLOCK_GOOD:      u8 = 0x63;
const MAGIC_UNLOCK_FEAT:      u8 = 0x64;

/**
 * Message lengths
 */
const MSGLEN_UNLOCK_CHAL:     usize = LEN_NONCE + LEN_NONCE_SIG;

#[entry]
fn main() -> ! {
  let mut timer: u64 = 0;
  loop {
    timer += 1;
    if driverlib::uart_avail_board() {
      let data: u8 = driverlib::uart_readb_board();
      match data {
        MAGIC_UNLOCK_REQ => {
          if is_paired {
            log!("Paired fob: Received UNLOCK_REQ");
            board.led_blue.set_high().unwrap();
            unlock_start();
            board.led_blue.set_low().unwrap();
          }
        }
        // Add other magic bytes here
        _ => {
          log!("Received invalid magic byte: {:x?}", data);
        }
      }
    }
  }
}

fn unlock_start() {
  // generate a nonce
  // TODO: use temperature and timer to generate random nonce (Jake)
  let car_nonce: u64 = 44444444;
  let car_nonce_bytes = car_nonce.to_be_bytes();

  // Get car secret key
  let mut car_secret_words: [u8; LEN_CAR_SECRET] = [0; LEN_CAR_SECRET];
  eeprom_read(CARMEM_CAR_SECRET, &mut car_secret_words);
  let car_secret_bytes = bytes_to_words(&car_secret_words);
  let car_secret = SecretKey::from_bytes(&car_secret_bytes).unwrap();

  // Use the car secret key to sign the nonce
  let car_signed_nonce = car_secret.sign(&car_nonce_bytes).to_be_bytes();

	// send unlock chal and nonce to fob
  let mut unlock_chal_msg: [u8; 1 + MSGLEN_UNLOCK_CHAL] = [MAGIC_UNLOCK_CHAL, car_nonce_bytes, car_signed_nonce];
  // unlock_chal_msg[1..].copy_from_slice(&signed_nonce); // copy starting at index 1, leaving space of 
  uart_write_board(&unlock_chal_msg);
  log!("Paired fob: Sent unlock_chal_msg to paired fob");

	// receive response/check signed nonce
  

	// send unlock result (success/fail)

	// receive the features from fob

}
