use core::arch::asm;
use crate::vga::text_mod::out::print;
use crate::vga::text_mod::out::ColorCode;
use crate::vga::text_mod::out::Color;
use crate::interrupts::idt::register_interrupt_handler;
use crate::x86::io::{inb, outb};

#[no_mangle]
pub extern "C" fn keyboard_interrupt_handler() {
    // PS/2 klavye veri portu: 0x60
    let scancode: u8;
    unsafe {
        scancode = inb(0x60); // Klavyeden gelen scancode'u oku
    }

    // ASCII eşlemesi çok basit tutacağız
    let ch = match scancode {
        0x02 => '1',
        0x03 => '2',
        0x04 => '3',
        0x05 => '4',
        0x06 => '5',
        0x07 => '6',
        0x08 => '7',
        0x09 => '8',
        0x0A => '9',
        0x0B => '0',
        0x1E => 'a',
        0x30 => 'b',
        0x2E => 'c',
        0x20 => 'd',
        0x12 => 'e',
        0x21 => 'f',
        0x22 => 'g',
        0x23 => 'h',
        0x17 => 'i',
        0x24 => 'j',
        0x25 => 'k',
        0x26 => 'l',
        0x32 => 'm',
        0x31 => 'n',
        0x18 => 'o',
        0x19 => 'p',
        0x10 => 'q',
        0x13 => 'r',
        0x1F => 's',
        0x14 => 't',
        0x16 => 'u',
        0x2F => 'v',
        0x11 => 'w',
        0x2D => 'x',
        0x15 => 'y',
        0x2C => 'z',
        0x39 => ' ',
        _ => return, // Diğer scancode'ları ignore et
    };

    print("keybprest", ColorCode::new(Color::White, Color::Black));

    // PIC’e “interrupt tamamlandı” bildirimi (EOI)
    unsafe {
        outb(0x20, 0x20); // Master PIC'e EOI gönder
        asm!("iret", options(nostack, preserves_flags)); // Interrupt'ı tamamla
    }
}

pub fn init_keyboard() {
    unsafe {
        register_interrupt_handler(33, keyboard_interrupt_handler); // IRQ1 = IDT index 32 + 1 = 33
    }
}

