#![no_std]
#![no_main]

// use panic_halt as _;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

use tiva::{
    driverlib::{self, uart_read_host, uart_write_board, uart_writeb_board, uart_readb_board, eeprom_read, eeprom_write, uart_read_board, wait},
    log, setup_board, Board,
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
const LEN_CAR_ID:             usize = 1; // stored as 4 bytes in EEPROM
const LEN_FEAT:               usize = 1; // stored as 4 bytes in EEPROM
const LEN_FEAT_SIG:           usize = 64;

// in words (for accesing EEPROM)
const LENW_FOB_SECRET:        usize = LEN_FOB_SECRET / 4;
const LENW_FOB_PUBLIC:        usize = LEN_FOB_PUBLIC / 4;
const LENW_CAR_SECRET:        usize = LEN_CAR_SECRET / 4;
const LENW_CAR_PUBLIC:        usize = LEN_CAR_PUBLIC / 4;
const LENW_MAN_SECRET:        usize = LEN_MAN_SECRET / 4;
const LENW_MAN_PUBLIC:        usize = LEN_MAN_PUBLIC / 4;
const LENW_CAR_ID:            usize = 1;
const LENW_FEAT:              usize = 1;
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

/**
 * Magic Bytes
 */
const MAGIC_PAIR_REQ:         u8 = 0x40;
const MAGIC_PAIR_SYN:         u8 = 0x41;
const MAGIC_PAIR_ACK:         u8 = 0x42;
const MAGIC_PAIR_FIN:         u8 = 0x43;
const MAGIC_PAIR_RST:         u8 = 0x44;

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

// FOR TESTING ONLY
const FOB_IS_PAIRED: bool = true;

#[entry]
fn main() -> ! {
  let mut board: Board = setup_board();

  // log!("This is fob!");

  board.led_blue.set_high().unwrap(); // Turn on blue LED

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

  let timer: u64 = 0;

  // Read from host UART, log output, and write back to host UART
  loop {
    timer += 1;
    if driverlib::uart_avail_host() {
      let data: u8 = driverlib::uart_readb_host();
      match (data) {
        MAGIC_PAIR_REQ => {
          if (FOB_IS_PAIRED) {
            paired_fob_pairing();
          }
        }
        MAGIC_PAIR_SYN => {
          if (!FOB_IS_PAIRED) {
            unpaired_fob_pairing();
          }
        }
        // ... add other magic bytes here
        _ => {}
      }
    }
  }
  // loop{}
}

fn paired_fob_pairing() {
  // we just received a PAIR_REQ
  // 1. Read PIN attempt from UART
  let mut pin: [u8; LEN_PIN_ATTEMPT] = [0; LEN_PIN_ATTEMPT];
  uart_read_host(&mut pin); // may need to read newline char

  // 2. Send PAIR_SYN and PIN attempt to unpaired fob
  let mut pair_syn_msg: [u8; 1 + LEN_PIN_ATTEMPT] = [MAGIC_PAIR_SYN; 1 + LEN_PIN_ATTEMPT]; // PAIR_SYN + pin command
  pair_syn_msg[1..].copy_from_slice(&pin); // copy starting at index 1, leaving space of 
  uart_write_board(&pair_syn_msg);

  // 3. compute hash of salt + pin
  let mut salt: [u32; LENW_FOB_SALT] = [0; LENW_FOB_SALT];
  eeprom_read(&mut salt, FOBMEM_FOB_SALT);
  let salt_bytes: [u8; 4] = salt[0].to_ne_bytes();

  let mut salted_pin :[u8; 4 + LEN_PIN_ATTEMPT] = [0; 4 + LEN_PIN_ATTEMPT ];
  salted_pin.copy_from_slice(&salt_bytes[0..3]);
  salted_pin[3..].copy_from_slice(&pin);
  let result = p256_cortex_m4::sha256(&salted_pin[..]);
  // 4. wait 500ms
  wait(2_000_000);
  // 5. check pair_ack
  let mut eeprom_pin_hash: [u32; LENW_PIN_HASH] = [0; LENW_PIN_HASH];
  eeprom_read(&eeprom_pin_hash, FOBMEM_PIN_HASH);
  // 6. do flowchart
  if eeprom_pin_hash == salted_pin{

  }else{
     wait(2_000_000); // change to 400 ms or time 
  }
  
  
}

fn unpaired_fob_pairing() {
  // we just received a PAIR_SYN, so now we need to read the PIN
  // 1. read pin from uart
  let mut pin: [u8; LEN_PIN_ATTEMPT] = [0; LEN_PIN_ATTEMPT];
  uart_read_board(&mut pin);

  // 2. send pair ack to paired fob
  let mut pair_ack_msg: u8 = MAGIC_PAIR_ACK;
  uart_writeb_board(pair_ack_msg);

  // 3. receive pair_fin magic from paired fob
  let mut magic_pair_fin: u8 = 0;
  magic_pair_fin = uart_readb_board();

  if (magic_pair_fin != MAGIC_PAIR_FIN) {
    //idk
  }

  // 4. receive secret from paired fob
  let mut secret: [u8; LEN_FOB_SECRET_ENC] = [0; LEN_FOB_SECRET_ENC];
  uart_read_board(&mut secret);

  // 5. receive car_id from paired fob
  let mut car_id: [u8; LEN_CAR_ID] = [0; LEN_CAR_ID];
  uart_read_board(&mut car_id);
  
  


  
  
}
