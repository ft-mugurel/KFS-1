use crate::{vga::text_mod::out::print, x86::io::outw};
use core::arch::asm;

#[allow(dead_code)]
fn are_interrupts_enabled() -> bool {
    let flag: u32;
    unsafe {
        asm!(
            "pushf",         // Push EFLAGS register onto the stack
            "pop {0}",       // Pop EFLAGS into `flag`
            out(reg) flag
        );
    }

    // Check if the Interrupt Flag (bit 9) is set
    (flag & (1 << 9)) != 0
}

pub(crate) fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nostack, preserves_flags)); // Enable interrupts
    }
}

pub(crate) unsafe fn request_shutdown() -> ! {
    outw(0x604, 0x2000); // QEMU
    outw(0xB004, 0x2000); // Bochs
    outw(0x4004, 0x3400); // VirtualBox

    loop {
        core::arch::asm!("hlt");
    }
}

pub(crate) unsafe fn request_reboot() {
    print("Trying to reboot...\n");
    print("Try method 1: Keyboard Controller\n");
    asm!("outb %al, %dx", in("dx") 0x64u16, in("al") 0xFEu8, options(att_syntax));

    // Method 2: PCI Reset
    print("Try method 2: PCI Reset\n");
    asm!("outb %al, %dx", in("dx") 0xCF9, in("al") 0x06u8, options(att_syntax));

    // Method 3: QEMU specific shutdown (if configured)
    print("Try method 3: QEMU specific reset\n");
    asm!("outw %ax, %dx", in("dx") 0x604, in("ax") 0x2000, options(att_syntax));
}
