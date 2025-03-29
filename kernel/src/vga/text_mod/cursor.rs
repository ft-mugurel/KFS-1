use x86::io::{outb, inb};

const VGA_CMD_PORT: u16 = 0x3D4;
const VGA_DATA_PORT: u16 = 0x3D5;

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
pub fn move_cursor(x: u16, y: u16) {
    let position = y * 80 + x;

    unsafe {
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
