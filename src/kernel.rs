#![no_std]
#![no_main]

pub mod x86;
pub mod interrupts;
pub mod vga;
pub mod gdt;

use core::panic::PanicInfo;

use gdt::gdt::load_gdt;

use interrupts::keyboard::init::init_keyboard;
use interrupts::idt::init_idt;
use interrupts::pic::init_pic;
use interrupts::pic::enable_interrupts;

use vga::text_mod::out::{print, ColorCode};
use vga::text_mod::out::Color;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    load_gdt();
    init_idt();
    unsafe {init_pic()};
    init_keyboard();
    enable_interrupts();
    loop {}
}

use core::arch::asm;

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

