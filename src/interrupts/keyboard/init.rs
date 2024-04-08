use crate::vga::text_mod::out::print_char;
use crate::vga::text_mod::out::ColorCode;
use crate::vga::text_mod::out::Color;
use crate::interrupts::idt::register_interrupt_handler;
use crate::x86::io::outb;
use crate::interrupts::keyboard::caracter_map::*;


#[no_mangle]
pub extern "C" fn keyboard_interrupt_handler() {
    // Read scancode from PS/2 keyboard data port (0x60)
    let scancode: u8 = unsafe {
        let mut code: u8;
        core::arch::asm!("in al, dx", out("al") code, in("dx") 0x60);
        code
    };
    // siplit the scancode to press and relase
    let pressed = scancode & 0x80 == 0;
    let scancode = scancode & 0x7F;
    let released = scancode & 0x80 != 0;

    if scancode < 128 {
        if let Some(ch) = LOWER_CARACTER_MAP[scancode as usize] {
            print_char(ch, ColorCode::new(Color::White, Color::Black));
        }
    }

    // Send End of Interrupt (EOI) to master PIC
    unsafe {
        outb(0x20, 0x20);
    }
}

extern "C" {
    fn isr_keyboard(); // the ISR we defined in NASM
}


pub fn init_keyboard() {
    unsafe {
        register_interrupt_handler(33, isr_keyboard); // IRQ1 = IDT index 32 + 1 = 33
    }
}

