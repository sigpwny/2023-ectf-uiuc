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
        pub(super) fn get_temp_samples(data: *mut u32);
        pub(super) fn sleep_us(us: u32);
        pub(super) fn start_delay_timer_us(us: u32);
        pub(super) fn wait_delay_timer();
        pub(super) fn get_remaining_us_delay_timer() -> u32;
        pub(super) fn get_tick_timer() -> u64;
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

// Read bytes from the board 
pub fn uart_read_board(data: &mut [u8]){
    for byte in data {
        *byte = uart_readb_board();
    }
}

/// Write a byte to the host.
pub fn uart_writeb_host(data: u8) {
    unsafe {
        driverwrapper::uart_writeb_host(data);
    }
}

// Write bytes to the host
pub fn uart_write_host(data: & [u8]){
    for byte in data{
        uart_writeb_host(*byte);
    }
}

/// Write a byte to the board.
pub fn uart_writeb_board(data: u8) {
    unsafe {
        driverwrapper::uart_writeb_board(data);
    }
}
// Write bytes to the board
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

pub fn get_temp_samples(samples: &mut [u32; 8]) {
    unsafe {
        driverwrapper::get_temp_samples(samples.as_mut_ptr())
    }
}

/// Waits for approximately the number of microseconds provided.
pub fn sleep_us(us: u32) {
    assert_ne!(us, 0);
    unsafe { driverwrapper::sleep_us(us) }
}

/// Sets up the delay timer to trigger after the microseconds provided
pub fn start_delay_timer_us(us: u32) {
    unsafe { driverwrapper::start_delay_timer_us(us) }
}

/// Waits for delay timer to be completed
pub fn wait_delay_timer() {
    unsafe { driverwrapper::wait_delay_timer() }
}

/// Gets remaining time on delay timer
pub fn get_remaining_us_delay_timer() -> u32 {
    unsafe { driverwrapper::get_remaining_us_delay_timer() }
}

/// Returns counter from PIOSC from startup
pub fn get_tick_timer() -> u64 {
    unsafe { driverwrapper::get_tick_timer() }
}