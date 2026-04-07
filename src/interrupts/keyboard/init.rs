use crate::interrupts::idt::register_interrupt_handler;
use crate::interrupts::keyboard::character_map::*;
use crate::interrupts::keyboard::keycode::{decode_set1_scancode, KeyCode, KeyEvent, Modifiers};
use crate::vga::text_mod::cursor::{
    disable_cursor, enable_cursor, set_big_cursor, set_cursor_shape, set_small_cursor,
};
use crate::vga::text_mod::out::{
    move_cursor_down, move_cursor_left, move_cursor_right, move_cursor_up, print_char,
    scroll_view_down, scroll_view_up, switch_screen,
};
use crate::x86::io::{outb, outw};

static mut EXTENDED_SCANCODE: bool = false;
static mut MODIFIERS: Modifiers = Modifiers::empty();

const SCANCODE_EXTENDED_PREFIX: u8 = 0xE0;
const KEYBOARD_DATA_PORT: u16 = 0x60;
const PIC_MASTER_COMMAND_PORT: u16 = 0x20;
const PIC_EOI: u8 = 0x20;

const KEYBOARD_IRQ_VECTOR: u8 = 33;

fn handle_key_press(event: KeyEvent, modifiers: Modifiers) -> bool {
    if event.key == KeyCode::Delete && modifiers.ctrl() && modifiers.alt() {
        return true;
    }

    match event.key {
        KeyCode::ArrowUp => {
            if modifiers.shift() {
                scroll_view_up();
            } else {
                move_cursor_up();
            }
        }
        KeyCode::ArrowDown => {
            if modifiers.shift() {
                scroll_view_down();
            } else {
                move_cursor_down();
            }
        }
        KeyCode::ArrowLeft => move_cursor_left(),
        KeyCode::ArrowRight => move_cursor_right(),
        KeyCode::F1 => switch_screen(0),
        KeyCode::F2 => switch_screen(1),
        KeyCode::F3 => switch_screen(2),
        KeyCode::F4 => switch_screen(3),
        KeyCode::F5 => switch_screen(4),
        KeyCode::F6 => switch_screen(5),
        KeyCode::F7 => set_big_cursor(),
        KeyCode::F8 => set_small_cursor(),
        KeyCode::F9 => set_cursor_shape(0, 15),
        KeyCode::F10 => disable_cursor(),
        KeyCode::F11 => enable_cursor(),
        _ => {
            if !modifiers.has_text_blocking_modifier() {
                if let Some(ch) = keycode_to_char(event.key, modifiers) {
                    print_char(ch);
                }
            }
        }
    }

    false
}

fn shutdown_system() -> ! {
    unsafe {
        outw(0x604, 0x2000); // QEMU
        outw(0xB004, 0x2000); // Bochs
        outw(0x4004, 0x3400); // VirtualBox
    }

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[no_mangle]
pub extern "C" fn keyboard_interrupt_handler() {
    let mut should_shutdown = false;

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
            if let Some(event) = decode_set1_scancode(scancode, EXTENDED_SCANCODE) {
                let mut modifiers = MODIFIERS;
                modifiers.update_for_event(event);
                MODIFIERS = modifiers;
                if event.pressed {
                    should_shutdown = handle_key_press(event, modifiers);
                }
            }
            EXTENDED_SCANCODE = false;
        }
    }

    // Send End of Interrupt (EOI) to master PIC
    unsafe {
        outb(PIC_MASTER_COMMAND_PORT, PIC_EOI);
    }

    if should_shutdown {
        shutdown_system();
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
