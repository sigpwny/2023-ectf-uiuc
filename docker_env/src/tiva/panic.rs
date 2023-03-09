use crate::tiva::board;
use crate::log;
use core::panic::PanicInfo;

/// Required by the compiler.
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() -> ! {
    board::panic();
}

/// Required by the compiler.
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr1() -> ! {
    board::panic();
}

/// Required by modules that haven't been build with panic = "abort"
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    board::panic();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log!("PANIC: {:?}", _info);
    board::panic();
}
