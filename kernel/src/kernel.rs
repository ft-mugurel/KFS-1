#![no_std]
#![no_main]

use core::panic::PanicInfo;
pub mod vga;
use vga::text_mod::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    cursor::move_cursor(10, 0);
    cursor::set_big_cursor();
    out::print("Hello, World!", out::ColorCode::new(out::Color::Yellow, out::Color::Black));
    out::print("Hello, World!", out::ColorCode::new(out::Color::Yellow, out::Color::Black));
    out::print("Hello, World!", out::ColorCode::new(out::Color::Yellow, out::Color::Black));
    out::clear(out::ColorCode::new(out::Color::Yellow, out::Color::Black));
    loop {}
}
