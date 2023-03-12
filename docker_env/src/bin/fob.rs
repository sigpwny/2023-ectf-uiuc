#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

use tiva::{
  driverlib::*,
  log, setup_board, Board, words_to_bytes, bytes_to_words, Signer, Verifier, sha256
};

use p256_cortex_m4::{SecretKey, Signature, PublicKey};
use rand_chacha::rand_core::SeedableRng;

/**
 * EEPROM state addresses (specifically for fob)
 */
const FOBMEM_FOB_SECRET:      u32 = 0x100;
const FOBMEM_FOB_SECRET_ENC:  u32 = 0x120;
const FOBMEM_FOB_SALT:        u32 = 0x140;
const FOBMEM_PIN_HASH:        u32 = 0x160;
const FOBMEM_CAR_ID:          u32 = 0x200;
const FOBMEM_FEAT_1_SIG:      u32 = 0x240;
const FOBMEM_FEAT_2_SIG:      u32 = 0x280;
const FOBMEM_FEAT_3_SIG:      u32 = 0x2C0;
const FOBMEM_CAR_PUBLIC:      u32 = 0x300;
const FOBMEM_FOB_IS_PAIRED:   u32 = 0x400;

const FOBMEM_MSG_FEAT_3:      u32 = 0x700;
const FOBMEM_MSG_FEAT_2:      u32 = 0x740;
const FOBMEM_MSG_FEAT_1:      u32 = 0x780;
const FOBMEM_MSG_UNLOCK:      u32 = 0x7C0;

    
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

// Pairing specific state
const LEN_FOB_SECRET_ENC:     usize = 32;
const LEN_FOB_SALT:           usize = 12;
const LEN_PIN_HASH:           usize = 32;
const LEN_FOB_IS_PAIRED:      usize = 4;

const LENW_FOB_SECRET_ENC:    usize = LEN_FOB_SECRET_ENC / 4;
const LENW_FOB_SALT:          usize = LEN_FOB_SALT / 4;
const LENW_PIN_HASH:          usize = LEN_PIN_HASH / 4;
const LENW_FOB_IS_PAIRED:     usize = LEN_FOB_IS_PAIRED / 4;

/**
 * Temporary state lengths
 */
const LEN_PIN_ATTEMPT:        usize = 3;
const LEN_FEAT_NUM:           usize = 4; // value of 1, 2, or 3
const LENW_FEAT_NUM:          usize = LEN_FEAT_NUM / 4;

/**
 * Magic Bytes
 */
const MAGIC_PAIR_REQ:         u8 = 0x40;
const MAGIC_PAIR_SYN:         u8 = 0x41;
const MAGIC_PAIR_ACK:         u8 = 0x42;
const MAGIC_PAIR_FIN:         u8 = 0x43;
const MAGIC_PAIR_RST:         u8 = 0x44;

const MAGIC_ENAB_FEAT:        u8 = 0x50;

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
const MSGLEN_PAIR_REQ:        usize = LEN_PIN_ATTEMPT;
const MSGLEN_PAIR_SYN:        usize = LEN_PIN_ATTEMPT;
const MSGLEN_PAIR_FIN:        usize = LEN_FOB_SECRET_ENC + 
                                      LEN_CAR_ID + 
                                      (LEN_FEAT_SIG * 3) + 
                                      LEN_CAR_PUBLIC;
                                      // Features sent in index order (1, 2, 3)
const MSGLEN_UNLOCK_CHAL:     usize = LEN_NONCE + LEN_NONCE_SIG;
const MSGLEN_UNLOCK_RESP:     usize = LEN_NONCE + LEN_NONCE_SIG;
const MSGLEN_UNLOCK_FEAT:     usize = LEN_FEAT_SIG * 3;

#[entry]
fn main() -> ! {
  let mut board: Board = setup_board();

  loop {
    // TODO: add LED resets
    if read_sw_1() && is_paired() {
      request_unlock(&mut board);
    }
    if uart_avail_host() {
      let magic: u8 = uart_readb_host();
      match magic {
        MAGIC_PAIR_REQ => {
          if is_paired() {
            // log!("Paired fob: Received PAIR_REQ");
            board.led_blue.set_high().unwrap();
            paired_fob_pairing();
            board.led_blue.set_low().unwrap();
          } else {
            // log!("Unpaired fob: Received invalid PAIR_REQ");
            board.led_red.set_high().unwrap();
            uart_writeb_host(MAGIC_HOST_FAILURE);
            sleep_us(1_000_000);
            board.led_red.set_low().unwrap();
          }
        }
        MAGIC_ENAB_FEAT => {
          if is_paired() {
            // log!("Paired fob: Received ENAB_FEAT");
            board.led_green.set_high().unwrap();
            enable_feature();
            board.led_green.set_low().unwrap();
          } else {
            // log!("Unpaired fob: Received invalid ENAB_FEAT");
            board.led_red.set_high().unwrap();
            uart_writeb_host(MAGIC_HOST_FAILURE);
            sleep_us(1_000_000);
            board.led_red.set_low().unwrap();
          }
        }
        _ => {
          // log!("Received invalid magic byte from host: {:x?}", magic);
        }
      }
    }
    if uart_avail_board() {
      let magic: u8 = uart_readb_board();
      match magic {
        MAGIC_PAIR_SYN => {
          if !is_paired() {
            // log!("Unpaired fob: Received PAIR_SYN");
            board.led_blue.set_high().unwrap();
            unpaired_fob_pairing();
            board.led_blue.set_low().unwrap();
            if is_paired() {
              board.led_green.set_high().unwrap();
              uart_writeb_host(MAGIC_HOST_SUCCESS);
              sleep_us(1_000_000);
              board.led_green.set_low().unwrap();
            } else {
              // log!("Unpaired fob: Failed to pair");
              board.led_red.set_high().unwrap();
              uart_writeb_host(MAGIC_HOST_FAILURE);
              sleep_us(1_000_000);
              board.led_red.set_low().unwrap();
            }
          }
        }
        _ => {
          // log!("Received invalid magic byte from board: {:x?}", magic);
        }
      }
    }
  }
}

/// Handle PAIR_REQ
fn paired_fob_pairing() {
  // Setup delay timer for 1000ms
  start_delay_timer_us(1_000_000);

  // 1. Read PIN attempt from UART
  let mut pin: [u8; LEN_PIN_ATTEMPT] = [0; LEN_PIN_ATTEMPT];
  uart_read_host(&mut pin);
  // log!("Paired fob: PAIR_REQ PIN value: {:x?}", pin);

  // 2. Send PAIR_SYN and PIN attempt to unpaired fob
  let mut pair_syn_msg: [u8; 1 + LEN_PIN_ATTEMPT] = [MAGIC_PAIR_SYN; 1 + LEN_PIN_ATTEMPT];
  pair_syn_msg[1..].copy_from_slice(&pin);
  uart_write_board(&pair_syn_msg);
  log!("Paired fob: Sent PAIR_SYN to unpaired fob");

  // 3. Compute hash of FOB_SALT + PIN
  let mut salt_w: [u32; LENW_FOB_SALT] = [0; LENW_FOB_SALT];
  let mut salt: [u8; LEN_FOB_SALT] = [0; LEN_FOB_SALT];
  let mut salted_pin :[u8; LEN_FOB_SALT + 1 + LEN_PIN_ATTEMPT] = [0; LEN_FOB_SALT + 1 + LEN_PIN_ATTEMPT];
  eeprom_read(&mut salt_w, FOBMEM_FOB_SALT);
  words_to_bytes(&salt_w, &mut salt);
  salted_pin[..LEN_FOB_SALT].copy_from_slice(&salt);
  salted_pin[LEN_FOB_SALT + 1..].copy_from_slice(&pin);
  let saltpin_hash = sha256(&salted_pin[..]);
  let mut saltpin_hash_w: [u32; 8] = [0; 8];
  bytes_to_words(&saltpin_hash,&mut saltpin_hash_w);

  // Block for 800ms
  while get_remaining_us_delay_timer() > 200_000 {}

  // 4. Check PAIR_ACK
  if uart_avail_board() {
    let magic: u8 = uart_readb_board();
    match magic {
      MAGIC_PAIR_ACK => {
        // log!("Paired fob: Received PAIR_ACK");
      }
      _ => {
        // log!("Paired fob: Received invalid magic byte: {:x?}", magic);
        return
      }
    }
  } else {
    // log!("Paired fob: PAIR_ACK timeout, could not find unpaired fob");
    return
  }

  // 5. Compute hash equality
  let mut eeprom_pin_hash_w: [u32; LENW_PIN_HASH] = [0; LENW_PIN_HASH];
  eeprom_read(&mut eeprom_pin_hash_w, FOBMEM_PIN_HASH);
  if eeprom_pin_hash_w == saltpin_hash_w {
    // PIN is correct, transmit PAIR_FIN
    // log!("Paired fob: PIN is correct");

    let mut secret_enc_w: [u32; LENW_FOB_SECRET_ENC] = [0; LENW_FOB_SECRET_ENC];
    let mut car_id_w: [u32; LENW_CAR_ID] = [0; LENW_CAR_ID];
    let mut feature_sig1_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
    let mut feature_sig2_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
    let mut feature_sig3_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
    let mut car_public_w: [u32; LENW_CAR_PUBLIC] = [0; LENW_CAR_PUBLIC];

    let mut secret_enc: [u8; LEN_FOB_SECRET_ENC] = [0; LEN_FOB_SECRET_ENC];
    let mut car_id: [u8; LEN_CAR_ID] = [0; LEN_CAR_ID];
    let mut feature_sig1: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
    let mut feature_sig2: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
    let mut feature_sig3: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
    let mut car_public: [u8; LEN_CAR_PUBLIC] = [0; LEN_CAR_PUBLIC];

    eeprom_read(&mut secret_enc_w, FOBMEM_FOB_SECRET_ENC);
    eeprom_read(&mut car_id_w, FOBMEM_CAR_ID);
    eeprom_read(&mut feature_sig1_w, FOBMEM_FEAT_1_SIG);
    eeprom_read(&mut feature_sig2_w, FOBMEM_FEAT_2_SIG);
    eeprom_read(&mut feature_sig3_w, FOBMEM_FEAT_3_SIG);
    eeprom_read(&mut car_public_w, FOBMEM_CAR_PUBLIC);

    words_to_bytes(& secret_enc_w, &mut secret_enc);
    words_to_bytes(& car_id_w, &mut car_id);
    words_to_bytes(& feature_sig1_w, &mut feature_sig1);
    words_to_bytes(& feature_sig2_w, &mut feature_sig2);
    words_to_bytes(& feature_sig3_w, &mut feature_sig3);
    words_to_bytes(& car_public_w, &mut car_public);

    // XOR decrypt FOB_SECRET_ENC with PIN + FOB_SALT
    let mut secret: [u8; LEN_FOB_SECRET] = [0; LEN_FOB_SECRET];
    let mut pinned_salt :[u8; LEN_PIN_ATTEMPT + 1 + LEN_FOB_SALT] = [0; LEN_PIN_ATTEMPT + 1 + LEN_FOB_SALT];
    pinned_salt[..LEN_PIN_ATTEMPT].copy_from_slice(&pin);
    pinned_salt[LEN_PIN_ATTEMPT + 1..].copy_from_slice(&salt);
    let pinsalt_hash = sha256(&pinned_salt);
    for i in 0..LEN_FOB_SECRET {
      secret[i] = secret_enc[i] ^ pinsalt_hash[i];
    }

    // log!("secret {:x?}", secret);
    // log!("car_id {:x?}", car_id);
    // log!("feature_sig1 {:x?}", feature_sig1);
    // log!("feature_sig2 {:x?}", feature_sig2);
    // log!("feature_sig3 {:x?}", feature_sig3);
    // log!("car_public {:x?}", car_public);

    uart_writeb_board(MAGIC_PAIR_FIN);
    uart_write_board(&mut secret);
    uart_write_board(&mut car_id);
    uart_write_board(&mut feature_sig1);
    uart_write_board(&mut feature_sig2);
    uart_write_board(&mut feature_sig3);
    uart_write_board(&mut car_public);
    // log!("Paired fob: Sent PAIR_FIN to unpaired fob");
    wait_delay_timer();
  } else {
    // PIN is incorrect, block for 5 seconds and then send PAIR_RST
    wait_delay_timer();
    sleep_us(4_000_000);
    // log!("Paired fob: PIN is incorrect");
    uart_writeb_board(MAGIC_PAIR_RST);
    // log!("Paired fob: Sent PAIR_RST to unpaired fob");
    // log!("Paired fob: PAIR transaction failed");
    return
  }

  log!("Paired fob: PAIR transaction completed");
}

/// Handle PAIR_SYN
fn unpaired_fob_pairing() {
  // 1. Read PIN from UART
  let mut pin: [u8; LEN_PIN_ATTEMPT] = [0; LEN_PIN_ATTEMPT];
  uart_read_board(&mut pin);
  // log!("Unpaired fob: PAIR_SYN PIN value: {:x?}", pin);

  // 2. Send PAIR_ACK to paired fob
  let pair_ack_msg: u8 = MAGIC_PAIR_ACK;
  uart_writeb_board(pair_ack_msg);
  // log!("Unpaired fob: Sent PAIR_ACK to paired fob");

  let mut secret: [u8; LEN_FOB_SECRET] = [0; LEN_FOB_SECRET];
  let mut car_id: [u8; LEN_CAR_ID] = [0; LEN_CAR_ID];
  let mut feature_sig1: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut feature_sig2: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut feature_sig3: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut car_public: [u8; LEN_CAR_PUBLIC] = [0; LEN_CAR_PUBLIC];

  // 3. Receive PAIR_FIN magic from paired fob
  loop {
    if uart_avail_board() {
      let magic: u8 = uart_readb_board();
      match magic {
        MAGIC_PAIR_FIN => {
          // log!("Unpaired fob: Received PAIR_FIN");
          break;
        }
        MAGIC_PAIR_RST => {
          // log!("Unpaired fob: Received PAIR_RST");
          // log!("Unpaired fob: PAIR transaction failed");
          return
        }
        _ => {
          // log!("Unpaired fob: Received invalid magic byte: {:x?}", magic);
        }
      }
    }
  }

  // 4. Receive data from paired fob
  uart_read_board(&mut secret);
  uart_read_board(&mut car_id);
  uart_read_board(&mut feature_sig1);
  uart_read_board(&mut feature_sig2);
  uart_read_board(&mut feature_sig3);
  uart_read_board(&mut car_public);
  // log!("Unpaired fob: Received PAIR_FIN data from paired fob");

  // log!("secret {:x?}", secret);
  // log!("car_id {:x?}", car_id);
  // log!("feature_sig1 {:x?}", feature_sig1);
  // log!("feature_sig2 {:x?}", feature_sig2);
  // log!("feature_sig3 {:x?}", feature_sig3);
  // log!("car_public {:x?}", car_public);

  // 5. Convert from bytes to words
  let mut secret_w: [u32; LENW_FOB_SECRET] = [0; LENW_FOB_SECRET];
  let mut car_id_w: [u32; LENW_CAR_ID] = [0; LENW_CAR_ID];
  let mut feature_sig1_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  let mut feature_sig2_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  let mut feature_sig3_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  let mut car_public_w: [u32; LENW_CAR_PUBLIC] = [0; LENW_CAR_PUBLIC];

  bytes_to_words(&secret, &mut secret_w);
  bytes_to_words(&car_id, &mut car_id_w);
  bytes_to_words(&feature_sig1, &mut feature_sig1_w);
  bytes_to_words(&feature_sig2, &mut feature_sig2_w);
  bytes_to_words(&feature_sig3, &mut feature_sig3_w);
  bytes_to_words(&car_public, &mut car_public_w);

  // 6. Create new PIN hash by hashing FOB_SALT + PIN with SHA256
  let mut salt_w: [u32; LENW_FOB_SALT] = [0; LENW_FOB_SALT];
  let mut salt: [u8; LEN_FOB_SALT] = [0; LEN_FOB_SALT];
  let mut salted_pin :[u8; LEN_FOB_SALT + 1 + LEN_PIN_ATTEMPT] = [0; LEN_FOB_SALT + 1 + LEN_PIN_ATTEMPT];
  eeprom_read(&mut salt_w, FOBMEM_FOB_SALT);
  words_to_bytes(&salt_w, &mut salt);
  salted_pin[..LEN_FOB_SALT].copy_from_slice(&salt);
  salted_pin[LEN_FOB_SALT + 1..].copy_from_slice(&pin);
  let saltpin_hash = sha256(&salted_pin);
  let mut saltpin_hash_w: [u32; LENW_PIN_HASH] = [0; LENW_PIN_HASH];
  bytes_to_words(&saltpin_hash, &mut saltpin_hash_w);

  // 7. Create new FOB_SECRET_ENC by XOR encrypting FOB_SECRET with the SHA256 hash of PIN + FOB_SALT
  let mut secret_enc: [u8; LEN_FOB_SECRET_ENC] = [0; LEN_FOB_SECRET_ENC];
  let mut secret_enc_w: [u32; LENW_FOB_SECRET_ENC] = [0; LENW_FOB_SECRET_ENC];
  let mut pinned_salt: [u8; LEN_PIN_ATTEMPT + 1 + LEN_FOB_SALT] = [0; LEN_PIN_ATTEMPT + 1 + LEN_FOB_SALT];
  pinned_salt[..LEN_PIN_ATTEMPT].copy_from_slice(&pin);
  pinned_salt[LEN_PIN_ATTEMPT + 1..].copy_from_slice(&salt);
  let pinsalt_hash = sha256(&pinned_salt);
  for i in 0..LEN_FOB_SECRET {
    secret_enc[i] = secret[i] ^ pinsalt_hash[i];
  }
  bytes_to_words(&secret_enc, &mut secret_enc_w);

  // 8. Write to EEPROM
  eeprom_write(&secret_enc_w, FOBMEM_FOB_SECRET_ENC);
  eeprom_write(&secret_w, FOBMEM_FOB_SECRET);
  eeprom_write(&car_id_w, FOBMEM_CAR_ID);
  eeprom_write(&feature_sig1_w, FOBMEM_FEAT_1_SIG);
  eeprom_write(&feature_sig2_w, FOBMEM_FEAT_2_SIG);
  eeprom_write(&feature_sig3_w, FOBMEM_FEAT_3_SIG);
  eeprom_write(&car_public_w, FOBMEM_CAR_PUBLIC);
  eeprom_write(&saltpin_hash_w, FOBMEM_PIN_HASH);

  // 9. Set paired flag
  set_paired();

  // log!("Unpaired fob: PAIR transaction completed");
}

/// Handle SW1 button press to unlock car
fn request_unlock(board: &mut Board) {
  // This does not need to be random since it is used for signature padding
  let rng = rand_chacha::ChaChaRng::from_seed([0; 32]);

  log!("Fob: Sending UNLOCK_REQ to car");
  uart_writeb_board(MAGIC_UNLOCK_REQ);
  
  // Receive unlock challenge from car
  loop {
    if uart_avail_board() {
      let magic: u8 = uart_readb_board();
      match magic {
        MAGIC_UNLOCK_CHAL => {
          break;
        }
        _ => {
          log!("Fob: Received unexpected message from car");
          // TODO: timeout
        }
      }
    }
  }

  let mut unlock_chal_msg: [u8; MSGLEN_UNLOCK_CHAL] = [0; MSGLEN_UNLOCK_CHAL];
  uart_read_board(&mut unlock_chal_msg);
  log!("Fob: Received UNLOCK_CHAL from car");
  board.led_blue.set_high().unwrap();

  // Read nonce from message
  let mut car_nonce_b: [u8; LEN_NONCE] = [0; LEN_NONCE];
  car_nonce_b.copy_from_slice(&unlock_chal_msg[..LEN_NONCE]);
  // log!("Fob: Received nonce value: {:x?}", car_nonce_b);

  // Read nonce signature from message
  let mut car_nonce_sig_b: [u8; LEN_NONCE_SIG] = [0; LEN_NONCE_SIG];
  car_nonce_sig_b.copy_from_slice(&unlock_chal_msg[LEN_NONCE..LEN_NONCE + LEN_NONCE_SIG]);
  log!("Fob: Received nonce signature: {:x?}", car_nonce_sig_b);

  // Read car public key from EEPROM
  let mut car_public_w: [u32; LENW_CAR_PUBLIC] = [0; LENW_CAR_PUBLIC];
  let mut car_public_b: [u8; LEN_CAR_PUBLIC] = [0; LEN_CAR_PUBLIC];
  eeprom_read(&mut car_public_w, FOBMEM_CAR_PUBLIC);
  words_to_bytes(&car_public_w, &mut car_public_b);
  let car_public = PublicKey::from_untagged_bytes(&car_public_b).unwrap();

  // Verify nonce signature
  let car_nonce_sig = Signature::from_untagged_bytes(&car_nonce_sig_b).unwrap();
  if !car_public.verify(&car_nonce_b, &car_nonce_sig) {
    log!("Fob: Car nonce signature verification failed");
    board.led_blue.set_low().unwrap();
    board.led_red.set_high().unwrap();
    uart_writeb_board(MAGIC_UNLOCK_RST);
    return;
  }

  // Increment nonce to sign
  let mut car_nonce: u64 = u64::from_be_bytes(car_nonce_b);
  car_nonce += 1;
  let fob_nonce_b: [u8; 8] = car_nonce.to_be_bytes();

  // Read fob secret key from EEPROM
  let mut fob_secret_w: [u32; LENW_FOB_SECRET] = [0; LENW_FOB_SECRET];
  let mut fob_secret_b: [u8; LEN_FOB_SECRET] = [0; LEN_FOB_SECRET];
  eeprom_read(&mut fob_secret_w, FOBMEM_FOB_SECRET);
  words_to_bytes(&fob_secret_w, &mut fob_secret_b);
  let fob_secret = SecretKey::from_bytes(&fob_secret_b).unwrap();
      
  // Use the fob secret key to sign the nonce
  let fob_signed_nonce: [u8; 64] = fob_secret.sign(&fob_nonce_b, rng).to_untagged_bytes();
  
  // Send signed nonce to car
  let mut fob_signed_msg: [u8; 1 + MSGLEN_UNLOCK_RESP] = [MAGIC_UNLOCK_RESP; 1 + MSGLEN_UNLOCK_RESP];
  fob_signed_msg[1..1 + LEN_NONCE].copy_from_slice(&fob_nonce_b);
  fob_signed_msg[1 + LEN_NONCE..].copy_from_slice(&fob_signed_nonce);
  // log!("Fob: Sending nonce: {:x?}", fob_nonce_b);
  // log!("Fob: Sending nonce signature: {:x?}", fob_signed_nonce);
  uart_write_board(&fob_signed_msg);
  board.led_blue.set_low().unwrap();

  log!("Fob: Sent UNLOCK_RESP to car");
  
  // Receive UNLOCK_GOOD from car
  loop {
    if uart_avail_board() {
      let magic: u8 = uart_readb_board();
      match magic {
        MAGIC_UNLOCK_GOOD => {
          if is_paired() {
            log!("Fob: Received UNLOCK_GOOD");
            board.led_green.set_high().unwrap();
            unlock_send_features();
            board.led_green.set_low().unwrap();
            return;
          }
        }
        MAGIC_UNLOCK_RST => {
          log!("Fob: Received UNLOCK_RST");
          return;
        }
        _ => {
          log!("Fob: Received unexpected message from car");
        }
      }
    }
    // TODO: timeout
  }
}

/// Handle UNLOCK_GOOD
fn unlock_send_features() {
  // Read features from EEPROM
  let mut feature_sig1_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  let mut feature_sig2_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  let mut feature_sig3_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  eeprom_read(&mut feature_sig1_w, FOBMEM_FEAT_1_SIG);
  eeprom_read(&mut feature_sig2_w, FOBMEM_FEAT_2_SIG);
  eeprom_read(&mut feature_sig3_w, FOBMEM_FEAT_3_SIG);

  // Convert features to bytes
  let mut feature_sig1_b: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut feature_sig2_b: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut feature_sig3_b: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  words_to_bytes(&feature_sig1_w, &mut feature_sig1_b);
  words_to_bytes(&feature_sig2_w, &mut feature_sig2_b);
  words_to_bytes(&feature_sig3_w, &mut feature_sig3_b);

  // Send UNLOCK_FEAT to car
  uart_writeb_board(MAGIC_UNLOCK_FEAT);
  uart_write_board(&feature_sig1_b);
  uart_write_board(&feature_sig2_b);
  uart_write_board(&feature_sig3_b);
  log!("Fob: Sent UNLOCK_FEAT to car");
}

/// Handle ENAB_FEAT
fn enable_feature() {
  // 1. Read in data
  let mut car_id: [u8; LEN_CAR_ID] = [0; LEN_CAR_ID];
  let mut feat_num: [u8; LEN_FEAT_NUM] = [0; LEN_FEAT_NUM];
  let mut feat_sig: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  uart_read_host(&mut car_id);
  uart_read_host(&mut feat_num);
  uart_read_host(&mut feat_sig);
  // log!("Paired fob: ENAB_FEAT feature number: {:x?}", feat_num);
  // log!("Paired fob: ENAB_FEAT feature signature: {:x?}", feat_sig);

  // 2. Convert each data element to words
  let mut car_id_w: [u32; LENW_CAR_ID] = [0; LENW_CAR_ID];
  let mut feat_num_w: [u32; LENW_FEAT_NUM] = [0; LENW_FEAT_NUM];
  let mut feat_sig_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  bytes_to_words(&car_id, &mut car_id_w);
  bytes_to_words(&feat_num, &mut feat_num_w);
  bytes_to_words(&feat_sig, &mut feat_sig_w);

  // Use as big endian word for comparison
  let feat_num_w_be: u32 = feat_num_w[0].to_be();

  // Block for 800ms
  sleep_us(800_000);

  // 3. Write the feature signature to EEPROM at the provided index
  if feat_num_w_be == 1 {
    eeprom_write(&feat_sig_w, FOBMEM_FEAT_1_SIG);
  } else if feat_num_w_be == 2 {
    eeprom_write(&feat_sig_w, FOBMEM_FEAT_2_SIG);
  } else if feat_num_w_be == 3 {
    eeprom_write(&feat_sig_w, FOBMEM_FEAT_3_SIG);
  } else {
    log!("Paired fob: Invalid feature number provided");
    uart_writeb_host(MAGIC_HOST_FAILURE);
    return;
  }

  // log!("Paired fob: Feature enabled");
  uart_writeb_host(MAGIC_HOST_SUCCESS);
}

/// Check the paired flag in EEPROM. Returns true if paired, false if unpaired.
fn is_paired() -> bool {
  let mut pair_status: [u32; LENW_FOB_IS_PAIRED] = [0; LENW_FOB_IS_PAIRED];
  eeprom_read(&mut pair_status, FOBMEM_FOB_IS_PAIRED);
  pair_status[0] != 0
}

/// Set the paired flag in EEPROM to 1.
fn set_paired() {
  let mut pair_status: [u32; LENW_FOB_IS_PAIRED] = [1; LENW_FOB_IS_PAIRED];
  eeprom_write(&mut pair_status, FOBMEM_FOB_IS_PAIRED);
}
