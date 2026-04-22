#![no_std]
#![no_main]

#[export_name = "efi_main"]
pub extern "C" fn main(_h: *mut core::ffi::c_void, _st: *mut core::ffi::c_void) -> usize {
    0
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
