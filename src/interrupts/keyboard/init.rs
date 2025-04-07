use core::arch::asm;
use crate::vga::text_mod::out::print;
use crate::vga::text_mod::out::print_char;
use crate::vga::text_mod::out::ColorCode;
use crate::vga::text_mod::out::Color;
use crate::interrupts::idt::register_interrupt_handler;
use crate::x86::io::{inb, outb};

static SCANCODE_MAP: [Option<char>; 128] = {
    let mut map = [None; 128];
    map[0x02] = Some('1');
    map[0x03] = Some('2');
    map[0x04] = Some('3');
    map[0x05] = Some('4');
    map[0x06] = Some('5');
    map[0x07] = Some('6');
    map[0x08] = Some('7');
    map[0x09] = Some('8');
    map[0x0A] = Some('9');
    map[0x0B] = Some('0');
    map[0x10] = Some('q');
    map[0x11] = Some('w');
    map[0x12] = Some('e');
    map[0x13] = Some('r');
    map[0x14] = Some('t');
    map[0x15] = Some('y');
    map[0x16] = Some('u');
    map[0x17] = Some('i');
    map[0x18] = Some('o');
    map[0x19] = Some('p');
    map[0x1E] = Some('a');
    map[0x1F] = Some('s');
    map[0x20] = Some('d');
    map[0x21] = Some('f');
    map[0x22] = Some('g');
    map[0x23] = Some('h');
    map[0x24] = Some('j');
    map[0x25] = Some('k');
    map[0x26] = Some('l');
    map[0x2C] = Some('z');
    map[0x2D] = Some('x');
    map[0x2E] = Some('c');
    map[0x2F] = Some('v');
    map[0x30] = Some('b');
    map[0x31] = Some('n');
    map[0x32] = Some('m');
    map[0x39] = Some(' ');
    map
};

#[no_mangle]
pub extern "C" fn keyboard_interrupt_handler() {
    // Read scancode from PS/2 keyboard data port (0x60)
    let scancode: u8 = unsafe {
        let mut code: u8;
        core::arch::asm!("in al, dx", out("al") code, in("dx") 0x60);
        code
    };

    // Ignore key releases (high bit set = key release)
    if scancode < 128 {
        if let Some(ch) = SCANCODE_MAP[scancode as usize] {
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

