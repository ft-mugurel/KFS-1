use crate::vga::text_mod::out::{
    move_cursor_down, move_cursor_left, move_cursor_right, move_cursor_up, print_char,
    scroll_view_down, scroll_view_up, switch_screen,
};
use crate::interrupts::idt::register_interrupt_handler;
use crate::x86::io::outb;
use crate::interrupts::keyboard::caracter_map::*;

static mut EXTENDED_SCANCODE: bool = false;
static mut LEFT_SHIFT_PRESSED: bool = false;
static mut RIGHT_SHIFT_PRESSED: bool = false;

const SCANCODE_EXTENDED_PREFIX: u8 = 0xE0;
const SCANCODE_RELEASE_MASK: u8 = 0x80;
const SCANCODE_KEY_MASK: u8 = 0x7F;
const KEYBOARD_DATA_PORT: u16 = 0x60;
const PIC_MASTER_COMMAND_PORT: u16 = 0x20;
const PIC_EOI: u8 = 0x20;

const SCANCODE_LEFT_SHIFT: u8 = 0x2A;
const SCANCODE_RIGHT_SHIFT: u8 = 0x36;

const SCANCODE_F1: u8 = 0x3B;
const SCANCODE_F6: u8 = 0x40;

const SCANCODE_ARROW_UP: u8 = 0x48;
const SCANCODE_ARROW_DOWN: u8 = 0x50;
const SCANCODE_ARROW_LEFT: u8 = 0x4B;
const SCANCODE_ARROW_RIGHT: u8 = 0x4D;

const KEYBOARD_IRQ_VECTOR: u8 = 33;

fn shift_pressed() -> bool {
    unsafe { LEFT_SHIFT_PRESSED || RIGHT_SHIFT_PRESSED }
}


#[no_mangle]
pub extern "C" fn keyboard_interrupt_handler() {
    // Read scancode from PS/2 keyboard data port (0x60)
    let scancode: u8 = unsafe {
        let mut code: u8;
        core::arch::asm!("in al, dx", out("al") code, in("dx") KEYBOARD_DATA_PORT);
        code
    };

    unsafe {
        if scancode == SCANCODE_EXTENDED_PREFIX {
            EXTENDED_SCANCODE = true;
        } else {
            let is_release = (scancode & SCANCODE_RELEASE_MASK) != 0;
            let keycode = scancode & SCANCODE_KEY_MASK;

            if EXTENDED_SCANCODE {
                match keycode {
                    _ if !is_release => match keycode {
                        SCANCODE_ARROW_UP => {
                            if shift_pressed() {
                                scroll_view_up();
                            } else {
                                move_cursor_up();
                            }
                        }
                        SCANCODE_ARROW_DOWN => {
                            if shift_pressed() {
                                scroll_view_down();
                            } else {
                                move_cursor_down();
                            }
                        }
                        SCANCODE_ARROW_LEFT => move_cursor_left(),
                        SCANCODE_ARROW_RIGHT => move_cursor_right(),
                        _ => {}
                    },
                    _ => {}
                }
                EXTENDED_SCANCODE = false;
            } else {
                match keycode {
                    SCANCODE_LEFT_SHIFT => LEFT_SHIFT_PRESSED = !is_release,
                    SCANCODE_RIGHT_SHIFT => RIGHT_SHIFT_PRESSED = !is_release,
                    SCANCODE_F1..=SCANCODE_F6 if !is_release => {
                        switch_screen((keycode - SCANCODE_F1) as usize);
                    }
                    _ if !is_release => {
                        if let Some(ch) = LOWER_CARACTER_MAP[keycode as usize] {
                            print_char(ch);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Send End of Interrupt (EOI) to master PIC
    unsafe {
        outb(PIC_MASTER_COMMAND_PORT, PIC_EOI);
    }
}

extern "C" {
    fn isr_keyboard(); // the ISR we defined in NASM
}


pub fn init_keyboard() {
    unsafe {
        register_interrupt_handler(KEYBOARD_IRQ_VECTOR, isr_keyboard); // IRQ1 = IDT index 32 + 1 = 33
    }
}

