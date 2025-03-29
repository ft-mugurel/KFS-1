// Assuming you have these constants defined
const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

// Color constants
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[allow(dead_code)]
pub fn clear(color: ColorCode) {
    let blank = 0x20 | ((color.0 as u16) << 8);
    
    unsafe {
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let index = y * VGA_WIDTH + x;
                VGA_BUFFER.offset(index as isize).write_volatile(blank);
            }
        }
    }
}

#[allow(dead_code)]
pub fn print(str: &str, color: ColorCode) {
    for (i, &byte) in str.as_bytes().iter().enumerate() {
        let vga_char = (byte as u16) | (color.0 as u16) << 8;
        unsafe {
            VGA_BUFFER.offset(i as isize).write_volatile(vga_char);
        }
    }
}

#[allow(dead_code)]
pub fn scroll() {
    for x in 0..VGA_HEIGHT { 
        for  y in 0..VGA_WIDTH {
            unsafe {
                let index = x * VGA_WIDTH + y;
                let index2 = (x + 1) * VGA_WIDTH + y;
                let vga_char = VGA_BUFFER.offset(index2 as isize).read_volatile();
                VGA_BUFFER.offset(index as isize).write_volatile(vga_char);
            }
        }
    }
}
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
