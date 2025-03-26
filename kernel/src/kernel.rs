#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga_buffer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    vga_buffer::kprint("Hello, World!", vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black));
    vga_buffer::kclear(vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black));
    loop {}
}
