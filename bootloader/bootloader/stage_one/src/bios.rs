/// TODO Make this a crate
use core::{arch::asm, ffi::c_uchar};
/// Video registers: http://vitaly_filatov.tripod.com/ng/asm/asm_023.1.html
pub fn get_video_mode() -> u8 {
    let mode;
    unsafe { asm!("int 0x10", in("ah") 0xFu8, out("al") mode) }
    mode
}

//TODO Return result
pub fn get_controller_info(buffer_addr: u32) -> Option<&'static VbeInfoBlock> {
    unsafe {
        let ax: u16;
        asm!("
        mov es, {seg:x}
        mov di, {off:x}
        int 0x10
        ", in("ax") 0x4F00u16, seg=in(reg) seg(buffer_addr), off=in(reg) off(buffer_addr) as u16);
        asm!("", out("ax") ax);
        if ax != 0x4F00 {
            return None;
        }
    }
    Some(unsafe { &*(buffer_addr as *const VbeInfoBlock) })
}

pub fn seg(addr: u32) -> u16 {
    (addr >> 4) as u16
}
pub fn off(addr: u32) -> u8 {
    (addr & 0b1111) as u8
}

#[repr(packed)]
#[repr(C, packed)]
pub struct VbeInfoBlock {
    pub vbe_signature: [c_uchar; 4], // == "VESA"
    pub vbe_version: u16,            // == 0x0300 for VBE 3.0
    pub oem_string_ptr: [u16; 2],    // isa vbeFarPtr
    pub capabilities: [u8; 4],
    pub video_mode_ptr: [u16; 2], // isa vbeFarPtr
    pub total_memory: u16,        // as # of 64KB blocks
    pub reserved: [u8; 492],
}
#[repr(C, packed)]
struct VbeModeInfoStructure {
    attributes: u16, // deprecated, only bit 7 should be of interest to you, and it indicates the mode supports a linear frame buffer.
    window_a: u8,    // deprecated
    window_b: u8,    // deprecated
    granularity: u16, // deprecated; used while calculating bank numbers
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32, // deprecated; used to switch banks from protected mode without returning to real mode
    pitch: u16,        // number of bytes per horizontal line
    width: u16,        // width in pixels
    height: u16,       // height in pixels
    w_char: u8,        // unused...
    y_char: u8,        // ...
    planes: u8,
    bpp: u8,   // bits per pixel in this mode
    banks: u8, // deprecated; total number of banks in this mode
    memory_model: u8,
    bank_size: u8, // deprecated; size of a bank, almost always 64 KB but may be 16 KB...
    image_pages: u8,
    reserved0: u8,
    red_mask: u8,
    red_position: u8,
    green_mask: u8,
    green_position: u8,
    blue_mask: u8,
    blue_position: u8,
    reserved_mask: u8,
    reserved_position: u8,
    direct_color_attributes: u8,
    framebuffer: u32, // physical address of the linear frame buffer; write here to draw to the screen
    off_screen_mem_off: u32,
    off_screen_mem_size: u16, // size of memory in the framebuffer but not being displayed on the screen
    reserved1: [u8; 206],
}
