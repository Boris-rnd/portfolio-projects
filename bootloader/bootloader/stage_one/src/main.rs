#![no_std]
#![no_main]
#![cfg_attr(debug_assertions, allow(unused, dead_code))]

use core::arch::{asm, global_asm};

mod protected;
use protected::*;
mod bios;
mod disk;
use disk::*;

const BASE: u16 = 0x7c00;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(disk_number: u16) {
    print("Stage 1...\r\n");
    let info = bios::get_controller_info(BASE as u32+10*1000).unwrap();
    for c in info.vbe_signature {
        chr_print(c);
    }
    // Currently not working cuz not in unreal mode
    let load_addr = 0x00100000;
    #[cfg(debug_assertions)]
    print_partitions();
    let partition_size = unsafe { core::ptr::read((BASE + 446 + 16 + 12) as *const u8) } as u64; // TODO Support partition size of u64
    let partition_offset = unsafe { core::ptr::read((BASE + 446 + 16 + 8) as *const u8) } as u64; // TODO Support partition size of u64
    for s in 0..partition_size {
        let addr = (load_addr as u64 + s * 512);
        let start_lba = s + partition_offset;
        let dap =
            DiskAddressPacket::from_lba(start_lba, 1, (addr & 0b1111) as u16, (addr >> 4) as u16);
        unsafe {
            dap.perform_load(disk_number);
        }
    } //BASE+2048+512+512
    print("Jumping stage 2");
    unsafe { enter_protected() };
    unsafe {
        asm!(
            // align the stack
            "and esp, 0xffffff00",
            // push arguments
            "push {disk:e}",
            // push entry point address
            "push {entry_point:e}",
            disk = in(reg) disk_number as u32,
            entry_point = in(reg) load_addr as u32,
        );
        asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
        asm!(
            ".code32",

            // reload segment registers
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov es, {0}",
            "mov ss, {0}",

            // jump to third stage
            "pop {1}",
            "call {1}",

            out(reg) _,
            out(reg) _,
        );
    }
    // This panics cuz it's in 16bit, so it's working !!
    print("Protec...\r\n");

    loop {}
}

fn print_partitions() {
    print("Partitions: \r\n");
    for p in 0..4 {
        let partition_size = unsafe { core::ptr::read((BASE + 446 + 16*p + 12) as *const u8) } as u64; // TODO Support partition size of u64
        let partition_offset = unsafe { core::ptr::read((BASE + 446 + 16*p + 8) as *const u8) } as u64; // TODO Support partition size of u64
        chr_print(b'-');
        chr_print(partition_size as u8 + b'0');
        chr_print(partition_offset as u8 + b'0');
        chr_print(b'\r');
        chr_print(b'\n');
    }
}
#[inline(always)]
fn chr_print(chr: u8) {
    unsafe {
        asm!("int 0x10", 
             in("al") chr, 
             in("ah") 0x0eu8
        );
    }
}
fn print(s: &str) {
    for c in s.chars() {
        chr_print(c as u8)
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    print("PANIC");
    loop {}
}
