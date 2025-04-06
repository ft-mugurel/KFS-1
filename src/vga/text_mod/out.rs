use super::cursor::{move_cursor, set_cursor, set_cursor_x, CURSOR};

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
        if byte == b'\n' {
            set_cursor_x(0);
            move_cursor(0, 1);
            continue;
        }
        unsafe {
            VGA_BUFFER.offset((CURSOR.y * (VGA_WIDTH as u16) + CURSOR.x) as isize).write_volatile(vga_char);
        }
        move_cursor(1, 0);
        if i >= VGA_WIDTH {
            move_cursor(0, 1);
        }
    }
}

#[allow(dead_code)]
pub fn newline() {
    unsafe {
        let mut index = VGA_WIDTH * (VGA_HEIGHT - 1);
        while index < VGA_WIDTH * VGA_HEIGHT {
            VGA_BUFFER.offset(index as isize).write_volatile(0x20);
            index += 1;
        }
    }
}

#[allow(dead_code)]
pub fn scroll() {
    for y in 0..VGA_HEIGHT { 
        for x in 0..VGA_WIDTH {
            unsafe {
                let index = y * VGA_WIDTH + x;
                let index2 = (y + 1) * VGA_WIDTH + x;
                let vga_char = VGA_BUFFER.offset(index2 as isize).read_volatile();
                VGA_BUFFER.offset(index as isize).write_volatile(vga_char);
            }
        }
    }
    set_cursor_x(0);
    move_cursor(0, -1);
}
