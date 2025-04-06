use crate::interrupts::interrupts::register_interrupt_handler;
use core::arch::asm;

const KEYBOARD_IRQ: u8 = 33; // IRQ1 mapped to interrupt vector 33
const KEYBOARD_DATA_PORT: u16 = 0x60;
const PIC_EOI: u8 = 0x20;
const PIC1_COMMAND: u16 = 0x20;

pub fn init_keyboard() {
    unsafe {
        register_interrupt_handler(KEYBOARD_IRQ, keyboard_interrupt_handler);
    }
}

fn custom_format_scancode(buffer: &mut [u8], scancode: u8) -> usize {
    let mut index = 0;

    // Write "Scancode: " to the buffer
    let prefix = b"Scancode: ";
    for &byte in prefix {
        if index < buffer.len() {
            buffer[index] = byte;
            index += 1;
        }
    }

    // Convert scancode to ASCII decimal and write to the buffer
    let mut num = scancode;
    let mut digits = [0u8; 3]; // Maximum 3 digits for u8
    let mut digit_count = 0;

    loop {
        digits[digit_count] = b'0' + (num % 10) as u8;
        digit_count += 1;
        num /= 10;
        if num == 0 {
            break;
        }
    }

    // Write digits in reverse order
    for i in (0..digit_count).rev() {
        if index < buffer.len() {
            buffer[index] = digits[i];
            index += 1;
        }
    }

    // Write newline character
    if index < buffer.len() {
        buffer[index] = b'\n';
        index += 1;
    }

    index // Return the number of bytes written
}

extern "C" fn keyboard_interrupt_handler() {
    let scancode: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") KEYBOARD_DATA_PORT,
            out("al") scancode,
            options(nomem, nostack, preserves_flags)
        );
    }

    // Prepare a buffer for the formatted string
    let mut buffer = [0u8; 32];
    let len = custom_format_scancode(&mut buffer, scancode);

    // Print the formatted string
    crate::vga::text_mod::out::print(
        core::str::from_utf8(&buffer[..len]).unwrap_or("Invalid UTF-8"),
        crate::vga::text_mod::out::ColorCode::new(
            crate::vga::text_mod::out::Color::White,
            crate::vga::text_mod::out::Color::Black,
        ),
    );

    // Send End of Interrupt (EOI) to the PIC
    unsafe {
        asm!(
            "mov al, {eoi}",
            "out dx, al",
            in("dx") PIC1_COMMAND,
            eoi = const PIC_EOI,
            options(nomem, nostack, preserves_flags)
        );
    }
}
