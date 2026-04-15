pub mod io;

use core::arch::asm;

#[inline]
pub fn read_esp() -> u32 {
    let value: u32;
    unsafe {
        asm!("mov {0:e}, esp", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

#[inline]
pub fn read_ebp() -> u32 {
    let value: u32;
    unsafe {
        asm!("mov {0:e}, ebp", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}