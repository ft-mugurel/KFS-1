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

pub fn kclear(color: ColorCode) {
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
pub fn kprint(str: &str, color: ColorCode) {
    for (i, &byte) in str.as_bytes().iter().enumerate() {
        let vga_char = (byte as u16) | (color.0 as u16) << 8;
        unsafe {
            VGA_BUFFER.offset(i as isize).write_volatile(vga_char);
        }
    }
}

