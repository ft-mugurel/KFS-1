use super::cursor::{disable_cursor, enable_cursor, set_cursor};

pub const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
const VGA_SIZE: usize = VGA_WIDTH * VGA_HEIGHT;
const VIRTUAL_SCREENS: usize = 6;
const SCROLLBACK_LINES: usize = 200;

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
pub struct ColorCode(pub u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

pub static mut CURRENT_COLOR: ColorCode = ColorCode::new(Color::White, Color::Black);
static mut SCREEN_BUFFERS: [[u16; VGA_SIZE * SCROLLBACK_LINES]; VIRTUAL_SCREENS] =
    [[0; VGA_SIZE * SCROLLBACK_LINES]; VIRTUAL_SCREENS];
static mut SCREEN_CURSORS: [super::cursor::Cursor; VIRTUAL_SCREENS] =
    [super::cursor::Cursor { x: 0, y: 0 }; VIRTUAL_SCREENS];
static mut SCREEN_USED_LINES: [usize; VIRTUAL_SCREENS] = [1; VIRTUAL_SCREENS];
static mut SCREEN_VIEWPORTS: [usize; VIRTUAL_SCREENS] = [0; VIRTUAL_SCREENS];
static mut ACTIVE_SCREEN: usize = 0;

fn blank_cell() -> u16 {
    unsafe { (b' ' as u16) | ((CURRENT_COLOR.0 as u16) << 8) }
}

fn cell_index(line: usize, column: usize) -> usize {
    line * VGA_WIDTH + column
}

fn clear_buffer_line(screen_index: usize, line: usize) {
    let blank = blank_cell();
    unsafe {
        for column in 0..VGA_WIDTH {
            SCREEN_BUFFERS[screen_index][cell_index(line, column)] = blank;
        }
    }
}

fn shift_buffer_up(screen_index: usize) {
    unsafe {
        for line in 1..SCROLLBACK_LINES {
            for column in 0..VGA_WIDTH {
                let src = cell_index(line, column);
                let dst = cell_index(line - 1, column);
                SCREEN_BUFFERS[screen_index][dst] = SCREEN_BUFFERS[screen_index][src];
            }
        }

        clear_buffer_line(screen_index, SCROLLBACK_LINES - 1);

        if SCREEN_CURSORS[screen_index].y > 0 {
            SCREEN_CURSORS[screen_index].y -= 1;
        }
    }
}

fn visible_top_line(screen_index: usize) -> usize {
    unsafe {
        let used_lines = SCREEN_USED_LINES[screen_index].min(SCROLLBACK_LINES);
        let max_scroll = used_lines.saturating_sub(VGA_HEIGHT);
        let viewport = SCREEN_VIEWPORTS[screen_index].min(max_scroll);
        max_scroll.saturating_sub(viewport)
    }
}

fn render_screen(screen_index: usize) {
    unsafe {
        let top_line = visible_top_line(screen_index);
        let used_lines = SCREEN_USED_LINES[screen_index].min(SCROLLBACK_LINES);

        for row in 0..VGA_HEIGHT {
            let source_line = top_line + row;
            for column in 0..VGA_WIDTH {
                let value = if source_line < used_lines {
                    SCREEN_BUFFERS[screen_index][cell_index(source_line, column)]
                } else {
                    blank_cell()
                };
                VGA_BUFFER.offset(cell_index(row, column) as isize)
                    .write_volatile(value);
            }
        }
    }

    update_hardware_cursor(screen_index);
}

fn current_screen_index() -> usize {
    unsafe { ACTIVE_SCREEN }
}

fn current_cursor() -> super::cursor::Cursor {
    unsafe { SCREEN_CURSORS[ACTIVE_SCREEN] }
}

fn sync_screen_state(screen_index: usize) {
    unsafe {
        SCREEN_USED_LINES[screen_index] = SCREEN_USED_LINES[screen_index].min(SCROLLBACK_LINES);
        SCREEN_VIEWPORTS[screen_index] = SCREEN_VIEWPORTS[screen_index]
            .min(SCREEN_USED_LINES[screen_index].saturating_sub(VGA_HEIGHT));
    }
}

fn newline_with_scroll() {
    let screen_index = current_screen_index();
    unsafe {
        let cursor = &mut SCREEN_CURSORS[screen_index];
        cursor.x = 0;

        let next_line = usize::from(cursor.y) + 1;
        if next_line >= SCROLLBACK_LINES {
            shift_buffer_up(screen_index);
            cursor.y = (SCROLLBACK_LINES - 1) as u16;
        } else {
            cursor.y = next_line as u16;
        }

        let cursor_line = usize::from(cursor.y);
        if cursor_line + 1 > SCREEN_USED_LINES[screen_index] {
            SCREEN_USED_LINES[screen_index] = cursor_line + 1;
            clear_buffer_line(screen_index, cursor_line);
        }

        sync_screen_state(screen_index);
    }

    render_screen(screen_index);
}

fn backspace() {
    let screen_index = current_screen_index();
    unsafe {
        let cursor = &mut SCREEN_CURSORS[screen_index];
        if cursor.x > 0 {
            cursor.x -= 1;
        } else if cursor.y > 0 {
            cursor.y -= 1;
            cursor.x = (VGA_WIDTH - 1) as u16;
        } else {
            return;
        }

        let index = cell_index(usize::from(cursor.y), usize::from(cursor.x));
        SCREEN_BUFFERS[screen_index][index] = blank_cell();
        sync_screen_state(screen_index);
    }

    render_screen(screen_index);
}

fn move_cursor_to(x: usize, y: usize) {
    let screen_index = current_screen_index();
    unsafe {
        let cursor = &mut SCREEN_CURSORS[screen_index];
        cursor.x = x.min(VGA_WIDTH - 1) as u16;
        cursor.y = y.min(SCROLLBACK_LINES - 1) as u16;
        let cursor_line = usize::from(cursor.y);
        if cursor_line + 1 > SCREEN_USED_LINES[screen_index] {
            SCREEN_USED_LINES[screen_index] = cursor_line + 1;
        }
        sync_screen_state(screen_index);
    }

    render_screen(screen_index);
}

fn write_cell(screen_index: usize, line: usize, column: usize, value: u16) {
    unsafe {
        SCREEN_BUFFERS[screen_index][cell_index(line, column)] = value;
    }
}

fn write_byte(byte: u8) {
    let screen_index = current_screen_index();

    match byte {
        b'\n' => newline_with_scroll(),
        b'\r' => unsafe {
            SCREEN_CURSORS[screen_index].x = 0;
            render_screen(screen_index);
        },
        0x08 => backspace(),
        b'\t' => {
            for _ in 0..4 {
                write_byte(b' ');
            }
        }
        byte => {
            let cursor = current_cursor();
            let vga_char = (byte as u16) | ((unsafe { CURRENT_COLOR.0 } as u16) << 8);
            write_cell(
                screen_index,
                usize::from(cursor.y),
                usize::from(cursor.x),
                vga_char,
            );

            unsafe {
                SCREEN_CURSORS[screen_index].x = SCREEN_CURSORS[screen_index].x.saturating_add(1);
                if usize::from(SCREEN_CURSORS[screen_index].x) >= VGA_WIDTH {
                    newline_with_scroll();
                } else {
                    sync_screen_state(screen_index);
                    render_screen(screen_index);
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn change_color(color: ColorCode) {
    unsafe {
        CURRENT_COLOR.0 = color.0;
    }
}

#[allow(dead_code)]
pub fn clear() {
    let screen_index = current_screen_index();
    unsafe {
        for line in 0..SCROLLBACK_LINES {
            clear_buffer_line(screen_index, line);
        }
        SCREEN_CURSORS[screen_index] = super::cursor::Cursor { x: 0, y: 0 };
        SCREEN_USED_LINES[screen_index] = 1;
        SCREEN_VIEWPORTS[screen_index] = 0;
    }
    render_screen(screen_index);
}

#[allow(dead_code)]
pub fn print(str: &str) {
    for &byte in str.as_bytes() {
        write_byte(byte);
    }
}

#[allow(dead_code)]
pub fn print_char(c: char) {
    let byte = if (c as u32) <= 0xFF { c as u8 } else { b'?' };
    write_byte(byte);
}

#[allow(dead_code)]
pub fn newline() {
    newline_with_scroll();
}

#[allow(dead_code)]
pub fn scroll() {
    newline_with_scroll();
}

#[allow(dead_code)]
pub fn move_cursor_left() {
    let screen_index = current_screen_index();
    unsafe {
        if SCREEN_CURSORS[screen_index].x > 0 {
            let cursor = SCREEN_CURSORS[screen_index];
            move_cursor_to(usize::from(cursor.x - 1), usize::from(cursor.y));
        }
    }
}

#[allow(dead_code)]
pub fn move_cursor_right() {
    let screen_index = current_screen_index();
    unsafe {
        if usize::from(SCREEN_CURSORS[screen_index].x) + 1 < VGA_WIDTH {
            let cursor = SCREEN_CURSORS[screen_index];
            move_cursor_to(usize::from(cursor.x + 1), usize::from(cursor.y));
        }
    }
}

#[allow(dead_code)]
pub fn move_cursor_up() {
    let screen_index = current_screen_index();
    unsafe {
        if SCREEN_CURSORS[screen_index].y > 0 {
            let cursor = SCREEN_CURSORS[screen_index];
            move_cursor_to(usize::from(cursor.x), usize::from(cursor.y - 1));
        }
    }
}

#[allow(dead_code)]
pub fn move_cursor_down() {
    let screen_index = current_screen_index();
    unsafe {
        if usize::from(SCREEN_CURSORS[screen_index].y) + 1 < SCREEN_USED_LINES[screen_index] {
            let cursor = SCREEN_CURSORS[screen_index];
            move_cursor_to(usize::from(cursor.x), usize::from(cursor.y + 1));
        }
    }
}

#[allow(dead_code)]
pub fn scroll_view_up() {
    let screen_index = current_screen_index();
    unsafe {
        let max_scroll = SCREEN_USED_LINES[screen_index].saturating_sub(VGA_HEIGHT);
        if SCREEN_VIEWPORTS[screen_index] < max_scroll {
            SCREEN_VIEWPORTS[screen_index] += 1;
        }
    }
    render_screen(screen_index);
}

#[allow(dead_code)]
pub fn scroll_view_down() {
    let screen_index = current_screen_index();
    unsafe {
        if SCREEN_VIEWPORTS[screen_index] > 0 {
            SCREEN_VIEWPORTS[screen_index] -= 1;
        }
    }
    render_screen(screen_index);
}

fn update_hardware_cursor(screen_index: usize) {
    unsafe {
        let top_line = visible_top_line(screen_index);
        let cursor = SCREEN_CURSORS[screen_index];
        let cursor_x = usize::from(cursor.x);
        let cursor_y = usize::from(cursor.y);

        if cursor_y >= top_line && cursor_y < top_line + VGA_HEIGHT {
            enable_cursor();
            set_cursor(cursor_x.min(VGA_WIDTH - 1) as u16, (cursor_y - top_line) as u16);
        } else {
            disable_cursor();
        }
    }
}

#[allow(dead_code)]
pub fn init_virtual_screens() {
    unsafe {
        for screen_index in 0..VIRTUAL_SCREENS {
            for line in 0..SCROLLBACK_LINES {
                clear_buffer_line(screen_index, line);
            }
            SCREEN_CURSORS[screen_index] = super::cursor::Cursor { x: 0, y: 0 };
            SCREEN_USED_LINES[screen_index] = 1;
            SCREEN_VIEWPORTS[screen_index] = 0;
        }
        ACTIVE_SCREEN = 0;
    }

    render_screen(0);
}

#[allow(dead_code)]
pub fn active_screen() -> usize {
    unsafe { ACTIVE_SCREEN }
}

#[allow(dead_code)]
pub fn switch_screen(screen_index: usize) {
    if screen_index >= VIRTUAL_SCREENS {
        return;
    }

    unsafe {
        ACTIVE_SCREEN = screen_index;
    }

    render_screen(screen_index);
}
