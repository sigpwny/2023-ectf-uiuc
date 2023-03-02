#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

use tiva::{
    driverlib::{self, uart_read_host, uart_avail_board, uart_write_board, uart_writeb_board, uart_readb_board, eeprom_read, eeprom_write, uart_read_board, wait},
    log, setup_board, Board, words_to_bytes, bytes_to_words
};

/**
 * EEPROM state addresses (specifically for fob)
 */
const FOBMEM_FOB_SECRET:      u32 = 0x100;
const FOBMEM_FOB_SECRET_ENC:  u32 = 0x120;
const FOBMEM_FOB_SALT:        u32 = 0x140;
const FOBMEM_PIN_HASH:        u32 = 0x160;
const FOBMEM_CAR_ID:          u32 = 0x200; // unused
const FOBMEM_FEAT_1:          u32 = 0x204;
const FOBMEM_FEAT_2:          u32 = 0x208;
const FOBMEM_FEAT_3:          u32 = 0x20C;
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
const LEN_FEATURE_NUM:        usize = 1;

/**
 * Magic Bytes
 */
const MAGIC_PAIR_REQ:         u8 = 0x40;
const MAGIC_PAIR_SYN:         u8 = 0x41;
const MAGIC_PAIR_ACK:         u8 = 0x42;
const MAGIC_PAIR_FIN:         u8 = 0x43;
const MAGIC_PAIR_RST:         u8 = 0x44;
const MAGIC_ENAB_FEAT:        u8 = 0x50;

/**
 * Message lengths
 */
const MSGLEN_PAIR_REQ:        usize = LEN_PIN_ATTEMPT;
const MSGLEN_PAIR_SYN:        usize = LEN_PIN_ATTEMPT;
const MSGLEN_PAIR_FIN:        usize = LEN_FOB_SECRET_ENC + 
                                      LEN_CAR_ID + 
                                      (LEN_FEAT * 3) + 
                                      (LEN_FEAT_SIG * 3) + 
                                      LEN_CAR_PUBLIC;
                                      // features sent in index order (1, 2, 3)

#[entry]
fn main() -> ! {
  let mut board: Board = setup_board();
  let mut is_paired: bool = true;   

  // log!("This is fob!");

  // Write "supersecret!" to EEPROM
  // Store "supersecret!" as a u32 array
  // let secret_1: u32 = 0x73757065;
  // let secret_2: u32 = 0x72736563;
  // let secret_3: u32 = 0x72657421;
  // let secret = [secret_1, secret_2, secret_3];
  // Write to EEPROM
  // driverlib::eeprom_write(&secret, 0);
  // Read from EEPROM
  // let mut read_out: [u32; 3] = [0; 3];
  // driverlib::eeprom_read(&mut read_out, 0);
  // log!("EEPROM readout: {:x?}", read_out);

  // Define string of characters to write to UART
  // let string: &[u8] = "This is fob, but over host serial!\n".as_bytes();
  // for i in 0..string.len() {
  //     driverlib::uart_writeb_host(string[i]);
  // }

  let mut timer: u64 = 0;

  // Read from host UART, log output, and write back to host UART
  loop {
    timer += 1;
    if driverlib::uart_avail_host() {
      let data: u8 = driverlib::uart_readb_host();
      match data {
        MAGIC_PAIR_REQ => {
          if is_paired {
            log!("Paired fob: Received PAIR_REQ");
            board.led_blue.set_high().unwrap();
            paired_fob_pairing();
            board.led_blue.set_low().unwrap();
          }
        }
        MAGIC_ENAB_FEAT => {
          if is_paired {
            log!("Paired fob: Received ENAB_FEAT");
            board.led_blue.set_high().unwrap();
            enable_feature();
            board.led_blue.set_low().unwrap();
          }
        }
        // Add other magic bytes here
        _ => {
          log!("Received invalid magic byte: {:x?}", data);
        }
      }
    }
    if uart_avail_board() {
      let data: u8 = uart_readb_board();
      match data {
        MAGIC_PAIR_SYN => {
          if !is_paired {
            log!("Unpaired fob: Received PAIR_SYN");
            board.led_blue.set_high().unwrap();
            unpaired_fob_pairing();
            is_paired = true;
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
  // loop{}
}

fn paired_fob_pairing() {
  // We just received a PAIR_REQ
  // 1. Read PIN attempt from UART
  let mut pin: [u8; LEN_PIN_ATTEMPT] = [0; LEN_PIN_ATTEMPT];
  uart_read_host(&mut pin); // may need to read newline char
  log!("Paired fob: PAIR_REQ PIN value: {:x?}", pin);

  // 2. Send PAIR_SYN and PIN attempt to unpaired fob
  let mut pair_syn_msg: [u8; 1 + LEN_PIN_ATTEMPT] = [MAGIC_PAIR_SYN; 1 + LEN_PIN_ATTEMPT]; // PAIR_SYN + pin command
  pair_syn_msg[1..].copy_from_slice(&pin); // copy starting at index 1, leaving space of 
  uart_write_board(&pair_syn_msg);
  log!("Paired fob: Sent PAIR_SYN to unpaired fob");

  // 3. Compute hash of FOB_SALT + PIN
  let mut salt: [u32; LENW_FOB_SALT] = [0; LENW_FOB_SALT];
  eeprom_read(&mut salt, FOBMEM_FOB_SALT);
  let mut salt_bytes: [u8; LEN_FOB_SALT] = [0; LEN_FOB_SALT];
  words_to_bytes(&salt, &mut salt_bytes);
  let mut salted_pin :[u8; LEN_FOB_SALT + 1 + LEN_PIN_ATTEMPT] = [0; LEN_FOB_SALT + 1 + LEN_PIN_ATTEMPT ];
  salted_pin[..LEN_FOB_SALT].copy_from_slice(&salt_bytes); // TODO: PANIC: PanicInfo { payload: Any { .. }, message: Some(source slice length (12) does not match destination slice length (15)), location: Location { file: "src\\bin\\fob.rs", line: 177, col: 14 }, can_unwind: true }
  salted_pin[LEN_FOB_SALT + 1..].copy_from_slice(&pin);
  let computed_pin_hash_b = p256_cortex_m4::sha256(&salted_pin[..]);
  let mut computed_pin_hash_w: [u32; 8] = [0;8];
  bytes_to_words(&computed_pin_hash_b,&mut computed_pin_hash_w);

  // 4. Wait 500ms
  wait(2_000_000); // TODO
  log!("Paired fob: Waiting 500ms for unpaired fob to respond");

  // 5. Check PAIR_ACK
  loop {
    // *timer += 1;
    if uart_avail_board() {
      let pair_ack: u8 = uart_readb_board();
      match pair_ack {
        MAGIC_PAIR_ACK => {
          log!("Paired fob: Received PAIR_ACK");
          break;
        }
        _ => {
          log!("Paired fob: Received invalid magic byte: {:x?}", pair_ack);
        }
      }
    }
    // TODO: Add timeout check "Could not find unpaired fob"
  }

  // Compute hash equality
  let mut eeprom_pin_hash: [u32; LENW_PIN_HASH] = [0; LENW_PIN_HASH];
  eeprom_read(&mut eeprom_pin_hash, FOBMEM_PIN_HASH);
  if eeprom_pin_hash == computed_pin_hash_w {
    log!("Paired fob: PIN is correct");
    uart_writeb_board(MAGIC_PAIR_FIN);
    log!("Paired fob: Sent PAIR_FIN to unpaired fob");
    // TODO: Write all the features etc for the final message
  } else {
    log!("Paired fob: PIN is incorrect");
    uart_writeb_board(MAGIC_PAIR_RST);
    log!("Paired fob: Sent PAIR_RST to unpaired fob");
    wait(2_000_000); // TODO: UART blocking (change to 400ms or time remaining)
    log!("Paired fob: PAIR transaction failed");
    return
  }

  log!("Paired fob: PAIR transaction completed")
}

fn unpaired_fob_pairing() {
  // We just received a PAIR_SYN, so now we need to read the PIN
  // 1. Read PIN from UART
  let mut pin: [u8; LEN_PIN_ATTEMPT] = [0; LEN_PIN_ATTEMPT];
  uart_read_board(&mut pin);
  log!("Unpaired fob: PAIR_SYN PIN value: {:x?}", pin);

  // 2. Send PAIR_ACK to paired fob
  let pair_ack_msg: u8 = MAGIC_PAIR_ACK;
  uart_writeb_board(pair_ack_msg);
  log!("Unpaired fob: Sent PAIR_ACK to paired fob");

  let mut secret: [u8; LEN_FOB_SECRET] = [0; LEN_FOB_SECRET];
  let mut car_id: [u8; LEN_CAR_ID] = [0; LEN_CAR_ID];
  let mut feature1: [u8; LEN_FEAT] = [0; LEN_FEAT];
  let mut feature2: [u8; LEN_FEAT] = [0; LEN_FEAT];
  let mut feature3: [u8; LEN_FEAT] = [0; LEN_FEAT];
  let mut feature_sig1: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut feature_sig2: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut feature_sig3: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  let mut car_public: [u8; LEN_CAR_PUBLIC] = [0; LEN_CAR_PUBLIC];

  // 3. Receive PAIR_FIN magic from paired fob
  loop {
    // *timer += 1;
    if uart_avail_board() {
      let pair_fin: u8 = uart_readb_board();
      match pair_fin {
        MAGIC_PAIR_FIN => {
          log!("Unpaired fob: Received PAIR_FIN");
          break;
        }
        MAGIC_PAIR_RST => {
          log!("Unpaired fob: Received PAIR_RST");
          log!("Unpaired fob: PAIR transaction failed");
          return;
        }
        _ => {
          log!("Unpaired fob: Received invalid magic byte: {:x?}", pair_fin);
        }
      }
    }
  }

  // 4. Receive secret, car_id, features, feature_sigs, and car_public from paired fob
  uart_read_board(&mut secret);
  uart_read_board(&mut car_id);
  uart_read_board(&mut feature1);
  uart_read_board(&mut feature2);
  uart_read_board(&mut feature3);
  uart_read_board(&mut feature_sig1);
  uart_read_board(&mut feature_sig2);
  uart_read_board(&mut feature_sig3);
  uart_read_board(&mut car_public);
  // TODO: implement a timeout for PAIR_FIN taking too long, but it's kinda annoying to do
  log!("Unpaired fob: Received PAIR_FIN data from paired fob");

  // 5. convert from bytes to words
  let mut secret_w: [u32; LENW_FOB_SECRET] = [0; LENW_FOB_SECRET];
  let mut car_id_w: [u32; LENW_CAR_ID] = [0; LENW_CAR_ID];
  let mut feature1_w: [u32; LENW_FEAT] = [0; LENW_FEAT];
  let mut feature2_w: [u32; LENW_FEAT] = [0; LENW_FEAT];
  let mut feature3_w: [u32; LENW_FEAT] = [0; LENW_FEAT];
  let mut feature_sig1_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  let mut feature_sig2_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  let mut feature_sig3_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  let mut car_public_w: [u32; LENW_CAR_PUBLIC] = [0; LENW_CAR_PUBLIC];

  bytes_to_words(&secret, &mut secret_w);
  bytes_to_words(&car_id, &mut car_id_w);
  bytes_to_words(&feature1, &mut feature1_w);
  bytes_to_words(&feature2, &mut feature2_w);
  bytes_to_words(&feature3, &mut feature3_w);
  bytes_to_words(&feature_sig1, &mut feature_sig1_w);
  bytes_to_words(&feature_sig2, &mut feature_sig2_w);
  bytes_to_words(&feature_sig3, &mut feature_sig3_w);
  bytes_to_words(&car_public, &mut car_public_w);

  // 6. TODO: Calculate new FOB_SECRET_ENC by combining the fob's own salt with the pin from earlier (we don't know how to do this yet)

  // 7. Create new pin hash by hashing the salt + pin with SHA256
  let mut salt: [u32; LENW_FOB_SALT] = [0; LENW_FOB_SALT];
  eeprom_read(&mut salt, FOBMEM_FOB_SALT);
  let mut salt_bytes: [u8; LEN_FOB_SALT] = [0; LEN_FOB_SALT];
  words_to_bytes(&salt, &mut salt_bytes);
  let mut salted_pin :[u8; LEN_FOB_SALT + 1 + LEN_PIN_ATTEMPT] = [0; LEN_FOB_SALT + 1 + LEN_PIN_ATTEMPT ];
  salted_pin[..LEN_FOB_SALT].copy_from_slice(&salt_bytes); // TODO: PANIC: PanicInfo { payload: Any { .. }, message: Some(source slice length (12) does not match destination slice length (15)), location: Location { file: "src\\bin\\fob.rs", line: 177, col: 14 }, can_unwind: true }
  salted_pin[LEN_FOB_SALT + 1..].copy_from_slice(&pin);
  let pin_hash_bytes = p256_cortex_m4::sha256(&salted_pin);
  let mut pin_hash: [u32; LENW_PIN_HASH] = [0; LENW_PIN_HASH];
  bytes_to_words(&pin_hash_bytes, &mut pin_hash);

  // 7. write to eeprom
  eeprom_write(&secret_w, FOBMEM_FOB_SECRET);
  eeprom_write(&car_id_w, FOBMEM_CAR_ID);
  eeprom_write(&feature1_w, FOBMEM_FEAT_1);
  eeprom_write(&feature2_w, FOBMEM_FEAT_2);
  eeprom_write(&feature3_w, FOBMEM_FEAT_3);
  eeprom_write(&feature_sig1_w, FOBMEM_FEAT_1_SIG);
  eeprom_write(&feature_sig2_w, FOBMEM_FEAT_2_SIG);
  eeprom_write(&feature_sig3_w, FOBMEM_FEAT_3_SIG);
  eeprom_write(&car_public_w, FOBMEM_CAR_PUBLIC);
  eeprom_write(&pin_hash, FOBMEM_PIN_HASH);
  
  log!("Unpaired fob: PAIR transaction completed")  
}

fn enable_feature() {
  // We just received ENAB_FEAT
  // 1. Read the feature index
  // LEN_FEATURE_NUM + LEN_FEATURE_NUM + LEN_FEAT_SIG
  let mut feat_idx: [u8; LEN_FEATURE_NUM] = [0; LEN_FEATURE_NUM]; // idx is also 1 byte
  uart_read_host(&mut feat_idx); // may need to read newline char
  log!("Paired fob: ENAB_FEAT feature index: {:x?}", feat_idx);

  // 2. check feature index, if it's not valid we reject request
  if (feat_idx[0] < 0x1 && feat_idx[0] > 0x3) {
    log!("Paired fob: Received invalid feature index: {:x?}", feat_idx[0]);
    return;
  }

  // 3. if feature index is good, read in the feature number
  let mut feat_num: [u8; LEN_FEATURE_NUM] = [0; LEN_FEATURE_NUM];
  uart_read_host(&mut feat_num); // may need to read newline char
  log!("Paired fob: ENAB_FEAT feature number: {:x?}", feat_num);

  // 4. also read in feature signature
  let mut feat_sig: [u8; LEN_FEAT_SIG] = [0; LEN_FEAT_SIG];
  uart_read_host(&mut feat_sig); // may need to read newline char
  log!("Paired fob: ENAB_FEAT feature signature: {:x?}", feat_sig);

  // 5. convert each item to words
  let mut feat_num_w: [u32; LENW_FEAT] = [0; LENW_FEAT];
  let mut feat_sig_w: [u32; LENW_FEAT_SIG] = [0; LENW_FEAT_SIG];
  bytes_to_words(&feat_num, &mut feat_num_w);
  bytes_to_words(&feat_sig, &mut feat_sig_w);

  // 6. write the new things to eeprom
  if (feat_idx[0] == 0x1) {
    eeprom_write(&feat_num_w, FOBMEM_FEAT_1);
    eeprom_write(&feat_sig_w, FOBMEM_FEAT_1_SIG);
    log!("Paired fob: Feature written to slot 1");
  } else if (feat_idx[0] == 0x2) {
    eeprom_write(&feat_num_w, FOBMEM_FEAT_2);
    eeprom_write(&feat_sig_w, FOBMEM_FEAT_2_SIG);
    log!("Paired fob: Feature written to slot 2");
  } else if (feat_idx[0] == 0x3) {
    eeprom_write(&feat_num_w, FOBMEM_FEAT_3);
    eeprom_write(&feat_sig_w, FOBMEM_FEAT_3_SIG);
    log!("Paired fob: Feature written to slot 3");
  }

  log!("Paired fob: Feature enable completed")
}
