use crate::{printk, x86::io::outw};
use core::arch::asm;

use crate::startup_config::power;

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

pub(crate) fn request_shutdown() -> ! {
    outw(power::QEMU_SHUTDOWN_PORT, power::QEMU_SHUTDOWN_VALUE); // QEMU
    outw(power::BOCHS_SHUTDOWN_PORT, power::BOCHS_SHUTDOWN_VALUE); // Bochs
    outw(power::VIRTUALBOX_SHUTDOWN_PORT, power::VIRTUALBOX_SHUTDOWN_VALUE); // VirtualBox

    loop {
        unsafe { asm!("hlt") };
    }
}

pub(crate) fn request_reboot() {
    printk!("Try method 1: Keyboard Controller\n");
    unsafe {
        asm!(
            "outb %al, %dx",
            in("dx") power::KEYBOARD_CONTROLLER_COMMAND_PORT,
            in("al") power::KEYBOARD_CONTROLLER_REBOOT,
            options(att_syntax)
        )
    };

    printk!("Try method 2: PCI Reset\n");
    unsafe {
        asm!(
            "outb %al, %dx",
            in("dx") power::PCI_RESET_PORT,
            in("al") power::PCI_RESET_VALUE,
            options(att_syntax)
        )
    };

    printk!("Try method 3: QEMU specific reset\n");
    unsafe {
        asm!(
            "outw %ax, %dx",
            in("dx") power::QEMU_SHUTDOWN_PORT,
            in("ax") power::QEMU_SHUTDOWN_VALUE,
            options(att_syntax)
        )
    };
}
