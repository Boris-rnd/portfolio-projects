#![no_std]
#![no_main]
#![cfg_attr(debug_assertions, allow(unused, dead_code))]

use core::arch::{asm, global_asm};

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(disk_number: u16) {
    // unsafe {asm!("sti")}
    let mut serial_port = unsafe { uart_16550::SerialPort::new(0x3F8) };
    serial_port.init();
    use core::fmt::Write;
    serial_port.send(b'H');
    serial_port.send(b'e');
    serial_port.send(b'l');
    unsafe {core::ptr::write_volatile(0xb8002 as *mut u8, b'g')}
    unsafe {core::ptr::write_volatile(0xb8003 as *mut u8, b'd')}
    // unsafe {core::ptr::write_volatile(0xb8001 as *mut u8, b'a')}
    // unsafe {core::ptr::write_volatile(0xb8002 as *mut u8, b'l')}

    // print("Stage 2...\r\n");

    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
