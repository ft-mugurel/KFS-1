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
use interrupts::utils::enable_interrupts;
use vga::text_mod::out::{print, Color, ColorCode};

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
    print("test", ColorCode::new(Color::White, Color::Black));
    loop {}
}

