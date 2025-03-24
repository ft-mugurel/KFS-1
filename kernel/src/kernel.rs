#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    kprint("Adem kernel sana girsin!");
    loop {}
}


pub fn kprint(str: &str) {
    let vga_buffer: *mut u8 = 0xb8000 as *mut u8;

    for (i, &byte) in str.as_bytes().iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xf;
        }
    }
}
