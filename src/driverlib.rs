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
        pub(super) fn check_switch() -> bool;
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
pub fn uart_avail_host() -> bool { unsafe { driverwrapper::uart_avail_host() } }

/// Check if the board has sent a byte.
pub fn uart_avail_board() -> bool { unsafe { driverwrapper::uart_avail_board() } }

/// Read a byte from the host.
pub fn uart_readb_host() -> i32 { unsafe { driverwrapper::uart_readb_host() } }

/// Read a byte from the board.
pub fn uart_readb_board() -> i32 { unsafe { driverwrapper::uart_readb_board() } }

/// Write a byte to the host.
pub fn uart_writeb_host(data: u8) { unsafe { driverwrapper::uart_writeb_host(data); } }

/// Write a byte to the board.
pub fn uart_writeb_board(data: u8) { unsafe { driverwrapper::uart_writeb_board(data); } }

/// Read from the EEPROM. Address must be a multiple of 4.
pub fn eeprom_read(data: &mut [u32], address: u32) {
    assert!(address + data.len() as u32 <= EEPROM_SIZE);
    unsafe {
        driverwrapper::eeprom_read(data.as_mut_ptr(), address, data.len() as u32);
    }
}

/// Write to the EEPROM. Address must be a multiple of 4.
pub fn eeprom_write(data: &[u32], address: u32) {
    assert!(address + data.len() as u32 <= EEPROM_SIZE);
    unsafe {
        driverwrapper::eeprom_write(data.as_ptr(), address, data.len() as u32);
    }
}

/// Check if the switch is pressed. Debounces if necessary.
pub fn check_switch() -> bool { unsafe { driverwrapper::check_switch() } }