use super::out;
use crate::x86::io::{inb, outb};

const VGA_CMD_PORT: u16 = 0x3D4;
const VGA_DATA_PORT: u16 = 0x3D5;

fn write_cursor_shape(start: u8, end: u8) {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        let cursor_start = inb(VGA_DATA_PORT);
        outb(VGA_DATA_PORT, (cursor_start & 0xE0) | (start & 0x1F));

        outb(VGA_CMD_PORT, 0x0B);
        let cursor_end = inb(VGA_DATA_PORT);
        outb(VGA_DATA_PORT, (cursor_end & 0xE0) | (end & 0x1F));
    }
}

#[allow(dead_code)]
pub fn set_big_cursor() {
    write_cursor_shape(0x00, 0x0F);
}

#[allow(dead_code)]
pub fn set_small_cursor() {
    write_cursor_shape(0x0E, 0x0F);
}

#[allow(dead_code)]
pub fn set_cursor_color(color: u8) {
    write_cursor_shape(color & 0x0F, 0x0F);
}

/*  Couldn't figure this one out. Also, not needed for now
#[allow(dead_code)]
pub fn set_cursor_blinking(blink: bool) {
    out::set_cursor_visible(out::current_screen_index(), blink);
}

#[allow(dead_code)]
pub fn set_cursor_blinking_rate(rate: u8) {
    write_cursor_shape(0x00, rate.min(0x0F));
} */

pub fn set_cursor_shape(start: u8, end: u8) {
    write_cursor_shape(start, end);
}

pub fn set_cursor(x: u16, y: u16) {
    let max_x = 79u16;
    let max_y = 24u16;
    let x = x.min(max_x);
    let y = y.min(max_y);
    let position = (y * 80 + x) as u16;
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

#[allow(dead_code)]
pub fn enable_cursor() {
    unsafe {
        outb(VGA_CMD_PORT, 0x0A);
        let cursor_start = inb(VGA_DATA_PORT);
        outb(VGA_DATA_PORT, cursor_start & !0x20);
    }
}

fn move_to(x: usize, y: usize) {
    let screen_index = out::current_screen_index();
    let clamped_x = x.min(out::VGA_WIDTH - 1);
    let clamped_y = y.min(out::SCROLLBACK_LINES - 1);
    out::set_cursor(screen_index, clamped_x, clamped_y);

    if clamped_y + 1 > out::used_lines_of_screen(screen_index) {
        out::set_used_lines(screen_index, clamped_y + 1);
    }

    out::sync_screen_state(screen_index);
    out::render_screen(screen_index);
}

pub(super) fn move_left() {
    let screen_index = out::current_screen_index();
    let cursor = out::cursor_of_screen(screen_index);
    if cursor.x > 0 {
        move_to(usize::from(cursor.x - 1), usize::from(cursor.y));
    }
}

pub(super) fn move_right() {
    let screen_index = out::current_screen_index();
    let cursor = out::cursor_of_screen(screen_index);
    if usize::from(cursor.x) + 1 < out::VGA_WIDTH {
        move_to(usize::from(cursor.x + 1), usize::from(cursor.y));
    }
}

pub(super) fn move_up() {
    let screen_index = out::current_screen_index();
    let cursor = out::cursor_of_screen(screen_index);
    if cursor.y > 0 {
        move_to(usize::from(cursor.x), usize::from(cursor.y - 1));
    }
}

pub(super) fn move_down() {
    let screen_index = out::current_screen_index();
    let cursor = out::cursor_of_screen(screen_index);
    if usize::from(cursor.y) + 1 < out::used_lines_of_screen(screen_index) {
        move_to(usize::from(cursor.x), usize::from(cursor.y + 1));
    }
}

pub(super) fn sync_hardware_cursor(screen_index: usize) {
    if !out::cursor_visible(screen_index) {
        disable_cursor();
        return;
    }

    let top_line = out::visible_top_line(screen_index);
    let cursor = out::cursor_of_screen(screen_index);
    let cursor_x = usize::from(cursor.x);
    let cursor_y = usize::from(cursor.y);

    if cursor_y >= top_line && cursor_y < top_line + out::VGA_HEIGHT {
        enable_cursor();
        set_cursor(
            cursor_x.min(out::VGA_WIDTH - 1) as u16,
            (cursor_y - top_line) as u16,
        );
    } else {
        disable_cursor();
    }
}
