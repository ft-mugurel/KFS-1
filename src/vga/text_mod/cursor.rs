use crate::x86::io::{inb, outb};

const VGA_CMD_PORT: u16 = 0x3D4;
const VGA_DATA_PORT: u16 = 0x3D5;

#[derive(Copy, Clone)]
pub struct Cursor {
    pub x: u16,
    pub y: u16,
}

pub static mut CURSOR: Cursor = Cursor { x: 0, y: 0 };

#[allow(dead_code)]
pub fn set_big_cursor() {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        outb(VGA_DATA_PORT, 0x00);
        outb(VGA_CMD_PORT, 0x0B);
        outb(VGA_DATA_PORT, 0x0F);
    }
}

#[allow(dead_code)]
pub fn set_small_cursor() {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        outb(VGA_DATA_PORT, 0x20);
        outb(VGA_CMD_PORT, 0x0B);
        outb(VGA_DATA_PORT, 0x07);
    }
}
#[allow(dead_code)]
pub fn set_cursor_color(color: u8) {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        let cursor_start = inb(VGA_DATA_PORT);
        outb(VGA_DATA_PORT, cursor_start | color);
    }
}
#[allow(dead_code)]
pub fn set_cursor_blinking(blink: bool) {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        let cursor_start = inb(VGA_DATA_PORT);
        if blink {
            // Bit 5 of register 0x0A disables the hardware cursor.
            // Clear it to keep the default hardware blink visible.
            outb(VGA_DATA_PORT, cursor_start & !0x20);
        } else {
            // Set bit 5 to hide the cursor when "blinking" is disabled.
            outb(VGA_DATA_PORT, cursor_start | 0x20);
        }
    }
}
#[allow(dead_code)]
pub fn set_cursor_blinking_rate(rate: u8) {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        let cursor_start = inb(VGA_DATA_PORT);
        outb(VGA_DATA_PORT, (cursor_start & 0xF8) | (rate & 0x07));
    }
}
#[allow(dead_code)]
pub fn set_cursor_shape(start: u8, end: u8) {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        outb(VGA_DATA_PORT, start);
        outb(VGA_CMD_PORT, 0x0B);
        outb(VGA_DATA_PORT, end);
    }
}

#[allow(dead_code)]
pub fn move_cursor(dx: i16, dy: i16) {
    unsafe {
        CURSOR.x = (CURSOR.x as i16 + dx).max(0) as u16;
        CURSOR.y = (CURSOR.y as i16 + dy).max(0) as u16;

        set_cursor(CURSOR.x, CURSOR.y);
    }
}

#[allow(dead_code)]
pub fn set_cursor(x: u16, y: u16) {
    let max_x = 79u16;
    let max_y = 24u16;
    let x = x.min(max_x);
    let y = y.min(max_y);
    let position = (y * 80 + x) as u16;
    unsafe {
        CURSOR.x = x;
        CURSOR.y = y;
        outb(VGA_CMD_PORT, 0x0E);
        outb(VGA_DATA_PORT, (position >> 8) as u8);
        outb(VGA_CMD_PORT, 0x0F);
        outb(VGA_DATA_PORT, (position & 0xFF) as u8);
    }
}

#[allow(dead_code)]
pub fn set_cursor_x(x: u16) {
    unsafe {
        let position = (CURSOR.y * 80 + x) as u16;
        CURSOR.x = x;
        outb(VGA_CMD_PORT, 0x0E);
        outb(VGA_DATA_PORT, (position >> 8) as u8);
        outb(VGA_CMD_PORT, 0x0F);
        outb(VGA_DATA_PORT, (position & 0xFF) as u8);
    }
}

#[allow(dead_code)]
pub fn set_cursor_y(y: u16) {
    unsafe {
        let position = (y * 80 + CURSOR.x) as u16;
        CURSOR.y = y;
        outb(VGA_CMD_PORT, 0x0E);
        outb(VGA_DATA_PORT, (position >> 8) as u8);
        outb(VGA_CMD_PORT, 0x0F);
        outb(VGA_DATA_PORT, (position & 0xFF) as u8);
    }
}

#[allow(dead_code)]
pub fn disable_cursor() {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        let cursor_start = inb(VGA_DATA_PORT);
        outb(VGA_DATA_PORT, cursor_start | 0x20);
    }
}

#[allow(dead_code)]
pub fn enable_cursor() {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        let cursor_start = inb(VGA_DATA_PORT);
        outb(VGA_DATA_PORT, cursor_start & !0x20);
    }
}
