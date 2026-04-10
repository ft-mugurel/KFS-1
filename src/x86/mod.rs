pub mod io;

use core::arch::asm;

#[inline]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nostack, preserves_flags));
    }
}

#[inline]
pub fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack));
    }
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}

#[inline]
pub fn read_cr0() -> u32 {
    let value: u32;
    unsafe {
        asm!("mov {0:e}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

#[inline]
pub fn write_cr0(value: u32) {
    unsafe {
        asm!("mov cr0, {0:e}", in(reg) value, options(nomem, nostack, preserves_flags));
    }
}

#[inline]
pub fn read_cr3() -> u32 {
    let value: u32;
    unsafe {
        asm!("mov {0:e}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

#[inline]
pub fn read_cr2() -> u32 {
    let value: u32;
    unsafe {
        asm!("mov {0:e}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

#[inline]
pub fn write_cr3(value: u32) {
    unsafe {
        asm!("mov cr3, {0:e}", in(reg) value, options(nomem, nostack, preserves_flags));
    }
}

#[inline]
pub fn enable_paging() {
    const CR0_PG: u32 = 1 << 31;
    let cr0 = read_cr0();
    write_cr0(cr0 | CR0_PG);
}
