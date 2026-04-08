use super::cursor;
use super::screen;
use core::fmt;
use core::fmt::Write;

#[derive(Copy, Clone)]
pub(super) struct ScreenCursor {
    pub x: u16,
    pub y: u16,
}

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
pub struct ColorCode(pub u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

pub fn active_screen_accepts_input() -> bool {
    screen::with_active_screen(|screen| screen.accepts_input)
}

pub fn set_screen_accepts_input(screen_index: usize, accepts_input: bool) {
    screen::with_screen_mut(screen_index, |screen| {
        screen.accepts_input = accepts_input;
    });
}

pub fn screen_accepts_input(screen_index: usize) -> bool {
    screen::with_screen(screen_index, |screen| screen.accepts_input).expect("Invalid screen index")
}

pub fn change_color(color: ColorCode) {
    screen::with_active_screen_mut(|screen| {
        screen.set_color(color);
    });
}

pub fn change_color_on(screen_index: usize, color: ColorCode) {
    screen::with_screen_mut(screen_index, |screen| {
        screen.set_color(color);
    });
}

pub fn clear() {
    screen::with_active_screen_mut(|screen| {
        for line in 0..screen::SCROLLBACK_LINES {
            screen::clear_buffer_line(screen, line);
        }
        screen.cursor = ScreenCursor { x: 0, y: 0 };
        screen.cursor_visible = true;
        screen.used_lines = 1;
        screen.viewport = 0;
        screen::render_screen(screen);
    });
}

pub fn print_on(screen_index: usize, str: &str) {
    screen::with_screen_mut(screen_index, |screen| {
        super::print::write_str_on(screen, str);
    });
}

pub fn print(str: &str) {
    print_on(screen::active_screen_index(), str);
}

pub fn write_fmt_on(screen_index: usize, args: fmt::Arguments<'_>) {
    screen::with_screen_mut(screen_index, |screen| {
        let _ = screen.write_fmt(args);
    });
}

pub fn write_fmt(args: fmt::Arguments<'_>) {
    write_fmt_on(screen::active_screen_index(), args);
}

pub fn print_char_on(screen_index: usize, c: char) {
    screen::with_screen_mut(screen_index, |screen| {
        super::print::write_char_on(screen, c);
    });
}

pub fn print_char(c: char) {
    print_char_on(screen::active_screen_index(), c);
}

pub fn newline_on(screen_index: usize) {
    screen::with_screen_mut(screen_index, |screen| {
        super::print::newline_on(screen);
    });
}

pub fn newline() {
    newline_on(screen::active_screen_index());
}

pub fn move_cursor_left() {
    cursor::move_left();
}

pub fn move_cursor_right() {
    cursor::move_right();
}

pub fn move_cursor_up() {
    cursor::move_up();
}

pub fn move_cursor_down() {
    cursor::move_down();
}

pub fn scroll_view_up() {
    screen::with_active_screen_mut(|screen| {
        let max_scroll = screen.used_lines.saturating_sub(screen::VGA_HEIGHT);
        if screen.viewport < max_scroll {
            screen.viewport += 1;
        }
        screen::render_screen(screen);
    });
}

pub fn scroll_view_down() {
    screen::with_active_screen_mut(|screen| {
        if screen.viewport > 0 {
            screen.viewport -= 1;
        }
        screen::render_screen(screen);
    });
}

pub fn init_virtual_screens() {
    screen::init();
}

pub fn switch_screen(screen_index: usize) {
    screen::set_active(screen_index);
}
