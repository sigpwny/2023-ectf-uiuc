#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

/**
 * EEPROM state addresses (specifically for fob)
 */


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
const LEN_NONCE:              usize = 8;
const LEN_NONCE_SIG:          usize = 64;

/**
 * Magic Bytes
 */
// start at 0x60
const MAGIC_UNLOCK_REQ:         u8 = 0x60;
const MAGIC_UNLOCK_CHAL:         u8 = 0x61;
const MAGIC_UNLOCK_RESP:         u8 = 0x62;
const MAGIC_UNLOCK_GOOD:         u8 = 0x63;
const MAGIC_UNLOCK_FEAT:         u8 = 0x64;

/**
 * Message lengths
 */


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

}
