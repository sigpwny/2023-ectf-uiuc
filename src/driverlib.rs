mod driverwrapper {
    #[link(name = "driverwrapper")]
    extern "C" {
        pub(super) fn init_system();
        pub(super) fn uart_avail_host() -> bool;
        pub(super) fn uart_avail_board() -> bool;
        pub(super) fn uart_readb_host() -> i32;
        pub(super) fn uart_readb_board() -> i32;
        pub(super) fn uart_writeb_host(data: u8);
        pub(super) fn uart_writeb_board(data: u8);
        pub(super) fn eeprom_read(data: *mut u32, address: u32, length: u32);
        pub(super) fn eeprom_write(data: *const u32, address: u32, length: u32);
        pub(super) fn read_sw_1() -> bool;
    }
}

const EEPROM_SIZE: u32 = 0x800; // 2K

/// Set up the system. This should be called after Board::new().
pub fn init_system() {
    unsafe {
        driverwrapper::init_system();
    }
}

/// Check if the host has sent a byte.
pub fn uart_avail_host() -> bool {
    unsafe { driverwrapper::uart_avail_host() }
}

/// Check if the board has sent a byte.
pub fn uart_avail_board() -> bool {
    unsafe { driverwrapper::uart_avail_board() }
}

/// Read a byte from the host.
pub fn uart_readb_host() -> u8 {
    // return as u8
    let ret: i32 = unsafe { driverwrapper::uart_readb_host() };
    ret as u8
}

// Read bytes from the host into an array. Only reads data.len() bytes
pub fn uart_read_host(data: &mut [u8]) {
    for byte in data {
        *byte = uart_readb_host();
    }
}

/// Read a byte from the board.
pub fn uart_readb_board() -> u8 {
    let ret: i32 = unsafe { driverwrapper::uart_readb_board() };
    ret as u8
}

/// Write a byte to the host.
pub fn uart_writeb_host(data: u8) {
    unsafe {
        driverwrapper::uart_writeb_host(data);
    }
}

/// Write a byte to the board.
pub fn uart_writeb_board(data: u8) {
    unsafe {
        driverwrapper::uart_writeb_board(data);
    }
}

pub fn uart_write_board(data: &[u8]) {
    for byte in data {
        uart_writeb_board(*byte);
    }
}
/// Read from the EEPROM. Address must be a multiple of 4.
pub fn eeprom_read(data: &mut [u32], address: u32) {
    if data.len() == 0 {
        return;
    }
    assert!(address + data.len() as u32 * 4 <= EEPROM_SIZE);
    unsafe {
        driverwrapper::eeprom_read(data.as_mut_ptr(), address, data.len() as u32 * 4);
    }
}

/// Write to the EEPROM. Address must be a multiple of 4.
pub fn eeprom_write(data: &[u32], address: u32) {
    if data.len() == 0 {
        return;
    }
    assert!(address + data.len() as u32 * 4 <= EEPROM_SIZE);
    unsafe {
        driverwrapper::eeprom_write(data.as_ptr(), address, data.len() as u32 * 4);
    }
}

/// Check if switch 1 is pressed. Returns true if pressed.
pub fn read_sw_1() -> bool {
    unsafe { driverwrapper::read_sw_1() }
}
