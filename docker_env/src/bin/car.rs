#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

use tiva::{
  driverlib::*,
  log, setup_board, Board, words_to_bytes, Signer, Verifier, get_combined_entropy, get_timer_entropy
};

use p256_cortex_m4::{SecretKey, Signature, PublicKey};
use rand_chacha::rand_core::{SeedableRng, RngCore, CryptoRng};


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
const LEN_FEAT_SIG:           usize = 64;
const LEN_FLAG:               usize = 64;

// in words (for accesing EEPROM)
const LENW_FOB_SECRET:        usize = LEN_FOB_SECRET / 4;
const LENW_FOB_PUBLIC:        usize = LEN_FOB_PUBLIC / 4;
const LENW_CAR_SECRET:        usize = LEN_CAR_SECRET / 4;
const LENW_CAR_PUBLIC:        usize = LEN_CAR_PUBLIC / 4;
const LENW_MAN_SECRET:        usize = LEN_MAN_SECRET / 4;
const LENW_MAN_PUBLIC:        usize = LEN_MAN_PUBLIC / 4;
const LENW_CAR_ID:            usize = LEN_CAR_ID / 4;
const LENW_FEAT_SIG:          usize = LEN_FEAT_SIG / 4;
const LENW_FLAG:              usize = LEN_FLAG / 4;

// Unlock specific state
const LEN_NONCE:              usize = 8; // 64-bit nonce
const LEN_NONCE_SIG:          usize = 64;

/**
 * Temporary state lengths
 */
const LEN_FEAT_NUM:           usize = 4; // value of 1, 2, or 3
const LENW_FEAT_NUM:          usize = LEN_FEAT_NUM / 4;

/**
 * Magic Bytes
 */
const MAGIC_UNLOCK_REQ:       u8 = 0x60;
const MAGIC_UNLOCK_CHAL:      u8 = 0x61;
const MAGIC_UNLOCK_RESP:      u8 = 0x62;
const MAGIC_UNLOCK_GOOD:      u8 = 0x63;
const MAGIC_UNLOCK_FEAT:      u8 = 0x64;
const MAGIC_UNLOCK_RST:       u8 = 0x69;

const MAGIC_HOST_SUCCESS:     u8 = 0xAA;
const MAGIC_HOST_FAILURE:     u8 = 0xBB;

/**
 * Message lengths
 */
const MSGLEN_UNLOCK_CHAL:     usize = LEN_NONCE + LEN_NONCE_SIG;
const MSGLEN_UNLOCK_RESP:     usize = LEN_NONCE + LEN_NONCE_SIG;
const MSGLEN_UNLOCK_FEAT:     usize = LEN_FEAT_SIG * 3;

#[entry]
fn main() -> ! {
  let mut board: Board = setup_board();

  // Seed RNG with entropy sources
  let entropy: [u8; 32] = get_combined_entropy();
  let mut timer_entropy: u64 = 0;
  let mut rng = rand_chacha::ChaChaRng::from_seed(entropy);

  loop {
    if uart_avail_board() {
      let magic: u8 = uart_readb_board();
      match magic {
        MAGIC_UNLOCK_REQ => {
          // log!("Car: Received UNLOCK_REQ");
          board.led_blue.set_high().unwrap();
          unlock_start(&mut rng, &mut board, &mut timer_entropy);
          board.led_blue.set_low().unwrap();
        }
        _ => {
          // log!("Received invalid magic byte: {:x?}", magic);
        }
      }
    }
  }
}

/// Handle UNLOCK_REQ
fn unlock_start(rng: &mut (impl CryptoRng + RngCore), board: &mut Board, timer_entropy: &mut u64) {
  // Start timeout timer for 500ms, need time to rx from fob
  start_delay_timer_us(500_000);

  // Update timer entropy
  let new_timer_entropy = get_timer_entropy();
  *timer_entropy ^= u64::from_ne_bytes(new_timer_entropy[0..8].try_into().unwrap());
  
  // Initialize car nonce with random value :) it's very random
  let mut car_nonce: u64 = rng.next_u64() ^ *timer_entropy;
  let car_nonce_b: [u8; 8] = car_nonce.to_be_bytes();

  // Get car secret key
  let mut car_secret_w: [u32; LENW_CAR_SECRET] = [0; LENW_CAR_SECRET];
  let mut car_secret_b: [u8; LEN_CAR_SECRET] = [0; LEN_CAR_SECRET];
  eeprom_read(&mut car_secret_w, CARMEM_CAR_SECRET);
  words_to_bytes(&car_secret_w, &mut car_secret_b);
  let car_secret = SecretKey::from_bytes(&car_secret_b).unwrap();

  // Use the car secret key to sign the nonce
  let car_signed_nonce: [u8; LEN_NONCE_SIG] = car_secret.sign(&car_nonce_b, rng).to_untagged_bytes();

  // Send unlock chal and nonce to fob
  let mut unlock_chal_msg: [u8; 1 + MSGLEN_UNLOCK_CHAL] = [MAGIC_UNLOCK_CHAL; 1 + MSGLEN_UNLOCK_CHAL];
  unlock_chal_msg[1..1 + LEN_NONCE].copy_from_slice(&car_nonce_b);
  unlock_chal_msg[1 + LEN_NONCE..].copy_from_slice(&car_signed_nonce);
  // log!("Car: Sending nonce: {:x?}", car_nonce_b);
  // log!("Car: Sending nonce signature: {:x?}", car_signed_nonce);
  uart_write_board(&unlock_chal_msg);
  // log!("Car: Sent UNLOCK_CHAL to paired fob");

  loop {
    if uart_avail_board() {
      let magic: u8 = uart_readb_board();
      match magic {
        MAGIC_UNLOCK_RESP => {
          break;
        }
        MAGIC_UNLOCK_RST => {
          // log!("Car: Received UNLOCK_RST");
          return;
        }
        _ => {
          // log!("Received invalid magic byte: {:x?}", magic);
        }
      }
    }
    // TODO: Add timeout
  }

  // Get UNLOCK_RESP data
  let mut unlock_resp_msg: [u8; MSGLEN_UNLOCK_RESP] = [0; MSGLEN_UNLOCK_RESP];
  uart_read_board(&mut unlock_resp_msg);
  // log!("Car: Received UNLOCK_RESP");

  // Read nonce signature from UNLOCK_RESP message
  let mut fob_signed_nonce: [u8; LEN_NONCE_SIG] = [0; LEN_NONCE_SIG];
  fob_signed_nonce.copy_from_slice(&unlock_resp_msg[LEN_NONCE..]);
  // log!("Car: Received nonce signature value: {:x?}", &fob_signed_nonce);

  // We check fob signature against car_nonce, NOT fob_nonce received from UART
  car_nonce += 1;
  let fob_nonce_b: [u8; 8] = car_nonce.to_be_bytes();

  // Get fob public key
  let mut fob_pubkey_w: [u32; LENW_FOB_PUBLIC] = [0; LENW_FOB_PUBLIC];
  let mut fob_pubkey_b: [u8; LEN_FOB_PUBLIC] = [0; LEN_FOB_PUBLIC];
  eeprom_read(&mut fob_pubkey_w, CARMEM_FOB_PUBLIC);
  words_to_bytes(&fob_pubkey_w, &mut fob_pubkey_b); 
  let fob_pubkey = PublicKey::from_untagged_bytes(&fob_pubkey_b).unwrap();

  // Load in the signature as a Signature type
  let fob_nonce_sig = Signature::from_untagged_bytes(&fob_signed_nonce).unwrap();
  // Verify the signature with the message and public key
  let fob_nonce_verified: bool = fob_pubkey.verify(&fob_nonce_b, &fob_nonce_sig);

  wait_delay_timer();

  if fob_nonce_verified {
    // yay unlock ze car
    // log!("Car: Unlocked!");
    board.led_blue.set_low().unwrap();
    board.led_green.set_high().unwrap();
    // Send unlock EEPROM message to UART host
    let mut unlock_msg_w: [u32; LENW_FLAG] = [0; LENW_FLAG];
    let mut unlock_msg_b: [u8; LEN_FLAG] = [0; LEN_FLAG];
    eeprom_read(&mut unlock_msg_w, CARMEM_MSG_UNLOCK);
    words_to_bytes(&unlock_msg_w, &mut unlock_msg_b);
    uart_write_host(&unlock_msg_b);

    unlock_request_features();
    board.led_green.set_low().unwrap();
  } else {
    // boo, bad signature
    // log!("Car: Bad signature, not unlocking");
    board.led_blue.set_low().unwrap();
    board.led_red.set_high().unwrap();
    sleep_us(4_500_000);
    uart_writeb_board(MAGIC_UNLOCK_RST);
    board.led_red.set_low().unwrap();
    return;
  }
}

/// Send UNLOCK_GOOD and handle UNLOCK_FEAT
fn unlock_request_features() {
  let mut feature_sig1_b: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut feature_sig2_b: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut feature_sig3_b: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];

  // Send UNLOCK_GOOD, signaling that we want to receive features
  // log!("Car: Sending UNLOCK_GOOD to fob");
  uart_writeb_board(MAGIC_UNLOCK_GOOD);

  // Wait for UNLOCK_FEAT from the fob
  loop {
    if uart_avail_board() {
      let magic: u8 = uart_readb_board();
      match magic {
        MAGIC_UNLOCK_FEAT => {
          break;
        }
        MAGIC_UNLOCK_RST => {
          // log!("Car: Received UNLOCK_RST");
          return;
        }
        _ => {
          // log!("Received invalid magic byte: {:x?}", magic);
        }
      }
    }
    // TODO: Add timeout
  }

  // Read UNLOCK_FEAT data
  uart_read_board(&mut feature_sig1_b);
  uart_read_board(&mut feature_sig2_b);
  uart_read_board(&mut feature_sig3_b);
  // log!("Car: Received UNLOCK_FEAT data");

  // Read in car ID from EEPROM
  let mut car_id_w: [u32; LENW_CAR_ID] = [0; LENW_CAR_ID];
  let mut car_id_b: [u8; LEN_CAR_ID] = [0; LEN_CAR_ID];
  eeprom_read(&mut car_id_w, CARMEM_CAR_ID);
  words_to_bytes(&car_id_w, &mut car_id_b);

  // Read in public key from EEPROM
  let mut man_public_w: [u32; LENW_MAN_PUBLIC] = [0; LENW_MAN_PUBLIC];
  let mut man_public_b: [u8; LEN_MAN_PUBLIC] = [0; LEN_MAN_PUBLIC];
  eeprom_read(&mut man_public_w, CARMEM_MAN_PUBLIC);
  words_to_bytes(&man_public_w, &mut man_public_b);
  
  // Load in the public key as a PublicKey type
  let man_public = PublicKey::from_untagged_bytes(&man_public_b).unwrap();

  // Load in signatures as Signature types
  let feature_sig1_res = Signature::from_untagged_bytes(&feature_sig1_b);
  let feature_sig2_res = Signature::from_untagged_bytes(&feature_sig2_b);
  let feature_sig3_res = Signature::from_untagged_bytes(&feature_sig3_b);

  // Concatenate car ID and feature numbers
  let feature1_b: [u8; LEN_FEAT_NUM] = [0x00, 0x00, 0x00, 0x01];
  let feature2_b: [u8; LEN_FEAT_NUM] = [0x00, 0x00, 0x00, 0x02];
  let feature3_b: [u8; LEN_FEAT_NUM] = [0x00, 0x00, 0x00, 0x03];
  let mut feat_pkg1: [u8; LEN_CAR_ID + LEN_FEAT_NUM] = [0; LEN_CAR_ID + LEN_FEAT_NUM];
  let mut feat_pkg2: [u8; LEN_CAR_ID + LEN_FEAT_NUM] = [0; LEN_CAR_ID + LEN_FEAT_NUM];
  let mut feat_pkg3: [u8; LEN_CAR_ID + LEN_FEAT_NUM] = [0; LEN_CAR_ID + LEN_FEAT_NUM];
  feat_pkg1[..LEN_CAR_ID].copy_from_slice(&car_id_b);
  feat_pkg1[LEN_CAR_ID..].copy_from_slice(&feature1_b);
  feat_pkg2[..LEN_CAR_ID].copy_from_slice(&car_id_b);
  feat_pkg2[LEN_CAR_ID..].copy_from_slice(&feature2_b);
  feat_pkg3[..LEN_CAR_ID].copy_from_slice(&car_id_b);
  feat_pkg3[LEN_CAR_ID..].copy_from_slice(&feature3_b);

  // Go through each feature, and validate signature of (CAR_ID + FEAT_NUM) using manufacturer public key
  // If correct, read the flag from EEPROM and send it to the host
  let mut feature_msg_w: [u32; LENW_FLAG] = [0; LENW_FLAG];
  let mut feature_msg_b: [u8; LEN_FLAG] = [0; LEN_FLAG];

  // Verify feature 1
  match feature_sig1_res {
    Ok(feature_sig1) => {
      if man_public.verify(&feat_pkg1, &feature_sig1) {
        eeprom_read(&mut feature_msg_w, CARMEM_MSG_FEAT_1);
        words_to_bytes(&feature_msg_w, &mut feature_msg_b);
        uart_write_host(&feature_msg_b);
        // log!("Car: Feature 1 flag sent");
      } else {
        // log!("Car: Feature 1 signature invalid");
      }
    }
    Err(_) => {
      // log!("Car: Feature 1 signature invalid");
    }
  }
  
  // Verify feature 2
  match feature_sig2_res {
    Ok(feature_sig2) => {
      if man_public.verify(&feat_pkg2, &feature_sig2) {
        eeprom_read(&mut feature_msg_w, CARMEM_MSG_FEAT_2);
        words_to_bytes(&feature_msg_w, &mut feature_msg_b);
        uart_write_host(&feature_msg_b);
        // log!("Car: Feature 2 flag sent");
      } else {
        // log!("Car: Feature 2 signature invalid");
      }
    }
    Err(_) => {
      // log!("Car: Feature 2 signature invalid");
    }
  }

  // Verify feature 3
  match feature_sig3_res {
    Ok(feature_sig3) => {
      if man_public.verify(&feat_pkg3, &feature_sig3) {
        eeprom_read(&mut feature_msg_w, CARMEM_MSG_FEAT_3);
        words_to_bytes(&feature_msg_w, &mut feature_msg_b);
        uart_write_host(&feature_msg_b);
        // log!("Car: Feature 3 flag sent");
      } else {
        // log!("Car: Feature 3 signature invalid");
      }
    }
    Err(_) => {
      // log!("Car: Feature 3 signature invalid");
    }
  }

  // log!("Car: All features processed");
}