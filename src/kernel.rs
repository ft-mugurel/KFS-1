#![no_std]
#![no_main]

use core::panic::PanicInfo;
pub mod vga;
pub mod interrupts;
use interrupts::keyboard::init::init_keyboard;
use vga::text_mod::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    init_keyboard(); // Initialize keyboard interrupts
    cursor::set_big_cursor();
    let asd = out::ColorCode::new(out::Color::Yellow, out::Color::Black);
    out::print("Hello\nHello", asd);
    loop {}
}
