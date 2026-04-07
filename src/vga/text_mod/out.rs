use super::cursor;

pub const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
const VIRTUAL_SCREENS_COUNT: usize = 6;
pub(super) const SCROLLBACK_LINES: usize = 200;

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

#[derive(Clone, Copy)]
pub(super) struct VirtualScreen {
    pub(super) buffer: [u16; VGA_WIDTH * SCROLLBACK_LINES],
    pub(super) cursor: ScreenCursor,
    pub(super) cursor_visible: bool,
    pub(super) used_lines: usize,
    pub(super) viewport: usize,
    pub(super) accepts_input: bool,
}

static mut VIRTUAL_SCREENS: [VirtualScreen; VIRTUAL_SCREENS_COUNT] = [VirtualScreen {
    buffer: [0; VGA_WIDTH * SCROLLBACK_LINES],
    cursor: ScreenCursor { x: 0, y: 0 },
    cursor_visible: true,
    used_lines: 1,
    viewport: 0,
    accepts_input: false,
}; VIRTUAL_SCREENS_COUNT];
pub(super) static mut CURRENT_COLOR: ColorCode = ColorCode::new(Color::White, Color::Black);
static mut ACTIVE_SCREEN_IDX: usize = 0;

fn blank_cell() -> u16 {
    unsafe { (b' ' as u16) | ((CURRENT_COLOR.0 as u16) << 8) }
}

pub(super) fn cell_index(line: usize, column: usize) -> usize {
    line * VGA_WIDTH + column
}

pub(super) fn clear_buffer_line(screen_index: usize, line: usize) {
    let blank = blank_cell();
    unsafe {
        for column in 0..VGA_WIDTH {
            VIRTUAL_SCREENS[screen_index].buffer[cell_index(line, column)] = blank;
        }
    }
}

pub(super) fn shift_buffer_up(screen_index: usize) {
    unsafe {
        for line in 1..SCROLLBACK_LINES {
            for column in 0..VGA_WIDTH {
                let src = cell_index(line, column);
                let dst = cell_index(line - 1, column);
                VIRTUAL_SCREENS[screen_index].buffer[dst] =
                    VIRTUAL_SCREENS[screen_index].buffer[src];
            }
        }

        clear_buffer_line(screen_index, SCROLLBACK_LINES - 1);

        if VIRTUAL_SCREENS[screen_index].cursor.y > 0 {
            VIRTUAL_SCREENS[screen_index].cursor.y -= 1;
        }
    }
}

pub(super) fn visible_top_line(screen_index: usize) -> usize {
    unsafe {
        let used_lines = VIRTUAL_SCREENS[screen_index]
            .used_lines
            .min(SCROLLBACK_LINES);
        let max_scroll = used_lines.saturating_sub(VGA_HEIGHT);
        let viewport = VIRTUAL_SCREENS[screen_index].viewport.min(max_scroll);
        max_scroll.saturating_sub(viewport)
    }
}

pub(super) fn render_screen(screen_index: usize) {
    unsafe {
        let top_line = visible_top_line(screen_index);
        let used_lines = VIRTUAL_SCREENS[screen_index]
            .used_lines
            .min(SCROLLBACK_LINES);

        for row in 0..VGA_HEIGHT {
            let source_line = top_line + row;
            for column in 0..VGA_WIDTH {
                let value = if source_line < used_lines {
                    VIRTUAL_SCREENS[screen_index].buffer[cell_index(source_line, column)]
                } else {
                    blank_cell()
                };
                VGA_BUFFER
                    .offset(cell_index(row, column) as isize)
                    .write_volatile(value);
            }
        }
    }

    sync_cursor(screen_index);
}

pub(super) fn render_cell_if_visible(screen_index: usize, line: usize, column: usize) {
    if line >= SCROLLBACK_LINES || column >= VGA_WIDTH {
        return;
    }

    unsafe {
        let top_line = visible_top_line(screen_index);
        if line < top_line || line >= top_line + VGA_HEIGHT {
            return;
        }

        let used_lines = VIRTUAL_SCREENS[screen_index]
            .used_lines
            .min(SCROLLBACK_LINES);
        let value = if line < used_lines {
            VIRTUAL_SCREENS[screen_index].buffer[cell_index(line, column)]
        } else {
            blank_cell()
        };

        let row = line - top_line;
        VGA_BUFFER
            .offset(cell_index(row, column) as isize)
            .write_volatile(value);
    }
}

pub(super) fn sync_cursor(screen_index: usize) {
    cursor::sync_hardware_cursor(screen_index);
}

pub(super) fn cursor_visible(screen_index: usize) -> bool {
    unsafe { VIRTUAL_SCREENS[screen_index].cursor_visible }
}

pub(super) fn current_screen_index() -> usize {
    unsafe { ACTIVE_SCREEN_IDX }
}

pub(super) fn cursor_of_screen(screen_index: usize) -> ScreenCursor {
    unsafe { VIRTUAL_SCREENS[screen_index].cursor }
}

pub(super) fn used_lines_of_screen(screen_index: usize) -> usize {
    unsafe { VIRTUAL_SCREENS[screen_index].used_lines }
}

pub(super) fn set_cursor(screen_index: usize, x: usize, y: usize) {
    unsafe {
        let cursor = &mut VIRTUAL_SCREENS[screen_index].cursor;
        cursor.x = x.min(VGA_WIDTH - 1) as u16;
        cursor.y = y.min(SCROLLBACK_LINES - 1) as u16;
    }
}

pub(super) fn set_cursor_x(screen_index: usize, x: usize) {
    unsafe {
        VIRTUAL_SCREENS[screen_index].cursor.x = x.min(VGA_WIDTH - 1) as u16;
    }
}

pub(super) fn set_used_lines(screen_index: usize, used_lines: usize) {
    unsafe {
        VIRTUAL_SCREENS[screen_index].used_lines = used_lines;
    }
}

pub(super) fn sync_screen_state(screen_index: usize) {
    unsafe {
        let screen = &mut VIRTUAL_SCREENS[screen_index];
        screen.used_lines = screen.used_lines.min(SCROLLBACK_LINES);
        screen.viewport = screen
            .viewport
            .min(screen.used_lines.saturating_sub(VGA_HEIGHT));
    }
}

pub(super) fn write_cell(screen_index: usize, line: usize, column: usize, value: u16) {
    unsafe {
        VIRTUAL_SCREENS[screen_index].buffer[cell_index(line, column)] = value;
    }
}

pub(super) fn write_blank_cell(screen_index: usize, line: usize, column: usize) {
    write_cell(screen_index, line, column, blank_cell());
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
        VIRTUAL_SCREENS[screen_index].cursor = ScreenCursor { x: 0, y: 0 };
        VIRTUAL_SCREENS[screen_index].cursor_visible = true;
        VIRTUAL_SCREENS[screen_index].used_lines = 1;
        VIRTUAL_SCREENS[screen_index].viewport = 0;
    }
    render_screen(screen_index);
}

pub fn print(str: &str) {
    super::print::write_str(str);
}

pub fn print_char(c: char) {
    super::print::write_char(c);
}

// #[allow(dead_code)]
pub fn newline() {
    super::print::newline();
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
    let screen_index = current_screen_index();
    unsafe {
        let max_scroll = VIRTUAL_SCREENS[screen_index]
            .used_lines
            .saturating_sub(VGA_HEIGHT);
        if VIRTUAL_SCREENS[screen_index].viewport < max_scroll {
            VIRTUAL_SCREENS[screen_index].viewport += 1;
        }
    }
    render_screen(screen_index);
}

pub fn scroll_view_down() {
    let screen_index = current_screen_index();
    unsafe {
        if VIRTUAL_SCREENS[screen_index].viewport > 0 {
            VIRTUAL_SCREENS[screen_index].viewport -= 1;
        }
    }
    render_screen(screen_index);
}

pub fn init_virtual_screens() {
    unsafe {
        for screen_index in 0..VIRTUAL_SCREENS_COUNT {
            for line in 0..SCROLLBACK_LINES {
                clear_buffer_line(screen_index, line);
            }
        }
        ACTIVE_SCREEN_IDX = 0;
    }
    render_screen(0);
}

#[allow(dead_code)]
pub fn active_screen() -> usize {
    unsafe { ACTIVE_SCREEN_IDX }
}

pub fn switch_screen(screen_index: usize) {
    if screen_index >= VIRTUAL_SCREENS_COUNT {
        return;
    }

    unsafe {
        ACTIVE_SCREEN_IDX = screen_index;
    }

    render_screen(screen_index);
}

pub fn active_screen_accepts_input() -> bool {
    unsafe { VIRTUAL_SCREENS[ACTIVE_SCREEN_IDX].accepts_input }
}
