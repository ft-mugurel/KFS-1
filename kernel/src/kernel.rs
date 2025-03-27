#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    vga::print("Hello, World!", vga::ColorCode::new(vga::Color::Yellow, vga::Color::Black));
    vga::clear(vga::ColorCode::new(vga::Color::Yellow, vga::Color::Black));
    vga::scroll();
    loop {}
}
