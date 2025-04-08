use core::fmt::{self, Write};
use crate::vga::text_mod::cursor::{move_cursor, set_cursor_x, CURSOR};
use crate::vga::text_mod::out::{VGA_BUFFER, VGA_WIDTH, ColorCode};

pub struct VgaWriter {
    pub color: ColorCode,
}

impl Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                set_cursor_x(0);
                move_cursor(0, 1);
                continue;
            }
            let vga_char = (byte as u16) | (self.color.0 as u16) << 8;
            unsafe {
                VGA_BUFFER
                    .offset((CURSOR.y * (VGA_WIDTH as u16) + CURSOR.x) as isize)
                    .write_volatile(vga_char);
            }
            move_cursor(1, 0);
        }
        Ok(())
    }
}

pub fn print_fmt(args: fmt::Arguments, color: ColorCode) {
    let mut writer = VgaWriter { color };
    writer.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        $crate::vga::text_mod::print_fmt(format_args!($($arg)*), ColorCode::new(Color::White, Color::Black));
    });
}
