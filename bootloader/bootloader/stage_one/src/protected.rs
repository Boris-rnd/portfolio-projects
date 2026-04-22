use core::arch::asm;

// / Enters unreal mode: https://wiki.osdev.org/Unreal_Mode
pub fn enter_unreal_mode() {
    // let ds: u16;
    // let ss: u16;
    // unsafe {
    //     asm!("mov {0:x}, ds", out(reg) ds, options(nomem, nostack, preserves_flags));
    //     asm!("mov {0:x}, ss", out(reg) ss, options(nomem, nostack, preserves_flags));
    // }
    // unsafe {asm!("cli")}
    // GDT.load();

    // // set protected mode bit
    // let cr0 = set_protected_mode_bit();

    // // load GDT
    // unsafe {
    //     asm!("mov {0}, 0x10", "mov ds, {0}", "mov ss, {0}", out(reg) _);
    // }

    // // unset protected mode bit again
    // write_cr0(cr0);

    // unsafe {
    //     asm!("mov ds, {0:x}", in(reg) ds, options(nostack, preserves_flags));
    //     asm!("mov ss, {0:x}", in(reg) ss, options(nostack, preserves_flags));
    //     asm!("sti");
    // }
}
/// https://wiki.osdev.org/Protected_mode
/// # Safety
/// Ensure A20 line is enabled
/// Ensure you jump to a 32 bit code segment after =)
#[inline(always)]
pub unsafe fn enter_protected() {
    unsafe { asm!("cli") }
    GDT.load();
    let mut cr0: u32;
    // Read CR0
    unsafe {
        asm!("mov {:e}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
    }
    // Sets the first bit
    cr0 |= 1;
    // Rewrites CR0
    unsafe { asm!("mov cr0, {:e}", in(reg) cr0, options(nostack, preserves_flags)) };
}

// FROM https://github.com/rust-osdev/bootloader/blob/main/bios/stage-2/src/protected_mode.rs
// Because I want to make smth fast
static GDT: GdtProtectedMode = GdtProtectedMode::new();

#[repr(C)]
pub struct GdtProtectedMode {
    zero: u64,
    code: u64,
    data: u64,
}

#[repr(C, packed(2))]
pub struct GdtPointer {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: *const GdtProtectedMode,
}
unsafe impl Send for GdtPointer {}
unsafe impl Sync for GdtPointer {}
impl GdtProtectedMode {
    const fn new() -> Self {
        let limit = {
            let limit_low = 0xffff;
            let limit_high = 0xf << 48;
            limit_high | limit_low
        };
        let access_common = {
            let present = 1 << 47;
            let user_segment = 1 << 44;
            let read_write = 1 << 41;
            present | user_segment | read_write
        };
        let protected_mode = 1 << 54;
        let granularity = 1 << 55;
        let base_flags = protected_mode | granularity | access_common | limit;
        let executable = 1 << 43;
        Self {
            zero: 0,
            code: base_flags | executable,
            data: base_flags,
        }
    }
    /// # Safety
    /// Ensure interrupts are disabled
    #[inline(always)]
    unsafe fn load(&'static self) {
        let pointer = GdtPointer {
            base: self,
            limit: (3 * core::mem::size_of::<u64>() - 1) as u16,
        };

        unsafe {
            asm!("lgdt [{}]", in(reg) &pointer, options(readonly, nostack, preserves_flags));
        }
    }
}
