use core::cell::UnsafeCell;
use core::fmt::Write;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::vga::text_mod::out::*;

use super::cursor;

pub const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
const VIRTUAL_SCREENS_COUNT: usize = 6;
pub(super) const SCROLLBACK_LINES: usize = 200;

#[derive(Clone, Copy)]
pub struct VirtualScreen {
    pub(super) buffer: [u16; VGA_WIDTH * SCROLLBACK_LINES],
    pub(super) cursor: ScreenCursor,
    pub(super) cursor_visible: bool,
    pub(super) used_lines: usize,
    pub(super) viewport: usize,
    pub(super) accepts_input: bool,
    pub(super) color: ColorCode,
    active: bool,
}

impl VirtualScreen {
    pub(super) fn set_color(&mut self, color: ColorCode) {
        self.color = color.clone();
    }
    fn blank_cell(&self) -> u16 {
        (b' ' as u16) | ((self.color.0 as u16) << 8)
    }
}

impl Write for VirtualScreen {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        super::print::write_str_on(self, s);
        Ok(())
    }
}

struct VirtualScreens(UnsafeCell<[VirtualScreen; VIRTUAL_SCREENS_COUNT]>);

impl VirtualScreens {
    const fn new() -> Self {
        Self(UnsafeCell::new(
            [VirtualScreen {
                buffer: [0; VGA_WIDTH * SCROLLBACK_LINES],
                cursor: ScreenCursor { x: 0, y: 0 },
                cursor_visible: true,
                used_lines: 1,
                viewport: 0,
                accepts_input: false,
                color: ColorCode::new(Color::White, Color::Black),
                active: false,
            }; VIRTUAL_SCREENS_COUNT],
        ))
    }

    fn with<R>(&self, screen_index: usize, f: impl FnOnce(&VirtualScreen) -> R) -> Option<R> {
        if screen_index >= VIRTUAL_SCREENS_COUNT {
            return None;
        }

        // Access is serialized by single-core execution and call-site discipline.
        let screens = unsafe { &*self.0.get() };
        Some(f(&screens[screen_index]))
    }

    fn with_mut<R>(
        &self,
        screen_index: usize,
        f: impl FnOnce(&mut VirtualScreen) -> R,
    ) -> Option<R> {
        if screen_index >= VIRTUAL_SCREENS_COUNT {
            return None;
        }

        // Access is serialized by single-core execution and call-site discipline.
        let screens = unsafe { &mut *self.0.get() };
        Some(f(&mut screens[screen_index]))
    }

    fn for_each_mut(&self, mut f: impl FnMut(&mut VirtualScreen)) {
        // Access is serialized by single-core execution and call-site discipline.
        let screens = unsafe { &mut *self.0.get() };
        for screen in screens.iter_mut() {
            f(screen);
        }
    }
}

unsafe impl Sync for VirtualScreens {}

static VIRTUAL_SCREENS: VirtualScreens = VirtualScreens::new();
static ACTIVE_SCREEN_IDX: AtomicUsize = AtomicUsize::new(0);

pub(super) fn with_screen<R>(
    screen_index: usize,
    f: impl FnOnce(&VirtualScreen) -> R,
) -> Option<R> {
    VIRTUAL_SCREENS.with(screen_index, f)
}

pub(super) fn with_screen_mut<R>(
    screen_index: usize,
    f: impl FnOnce(&mut VirtualScreen) -> R,
) -> Option<R> {
    VIRTUAL_SCREENS.with_mut(screen_index, f)
}

pub(super) fn cell_index(line: usize, column: usize) -> usize {
    line * VGA_WIDTH + column
}

pub(super) fn clear_buffer_line(screen: &mut VirtualScreen, line: usize) {
    let blank = screen.blank_cell();
    for column in 0..VGA_WIDTH {
        screen.buffer[cell_index(line, column)] = blank;
    }
}

pub(super) fn shift_buffer_up(screen: &mut VirtualScreen) {
    for line in 1..SCROLLBACK_LINES {
        for column in 0..VGA_WIDTH {
            let src = cell_index(line, column);
            let dst = cell_index(line - 1, column);
            screen.buffer[dst] = screen.buffer[src];
        }
    }

    clear_buffer_line(screen, SCROLLBACK_LINES - 1);

    if screen.cursor.y > 0 {
        screen.cursor.y -= 1;
    }
}

pub(super) fn visible_top_line_of(screen: &VirtualScreen) -> usize {
    let used_lines = screen.used_lines.min(SCROLLBACK_LINES);
    let max_scroll = used_lines.saturating_sub(VGA_HEIGHT);
    let viewport = screen.viewport.min(max_scroll);
    max_scroll.saturating_sub(viewport)
}

fn render_screen_buffer(screen: &VirtualScreen) {
    let top_line = visible_top_line_of(screen);
    let used_lines = screen.used_lines.min(SCROLLBACK_LINES);

    for row in 0..VGA_HEIGHT {
        let source_line = top_line + row;
        for column in 0..VGA_WIDTH {
            let value = if source_line < used_lines {
                screen.buffer[cell_index(source_line, column)]
            } else {
                screen.blank_cell()
            };
            unsafe {
                VGA_BUFFER
                    .offset(cell_index(row, column) as isize)
                    .write_volatile(value);
            }
        }
    }
}

pub(super) fn render_screen(screen: &mut VirtualScreen) {
    render_screen_buffer(screen);
    sync_cursor_of(screen);
}

pub(super) fn render_cell_if_visible_of(screen: &VirtualScreen, line: usize, column: usize) {
    if line >= SCROLLBACK_LINES || column >= VGA_WIDTH {
        return;
    }

    let top_line = visible_top_line_of(screen);
    if line < top_line || line >= top_line + VGA_HEIGHT {
        return;
    }

    let used_lines = screen.used_lines.min(SCROLLBACK_LINES);
    let value = if line < used_lines {
        screen.buffer[cell_index(line, column)]
    } else {
        screen.blank_cell()
    };

    let row = line - top_line;
    unsafe {
        VGA_BUFFER
            .offset(cell_index(row, column) as isize)
            .write_volatile(value);
    }
}

pub(super) fn active_screen_index() -> usize {
    ACTIVE_SCREEN_IDX.load(Ordering::Relaxed)
}

pub(super) fn with_active_screen<R>(f: impl FnOnce(&VirtualScreen) -> R) -> R {
    let screen_index = active_screen_index();
    with_screen(screen_index, f).expect("active screen index out of bounds")
}

pub(super) fn with_active_screen_mut<R>(f: impl FnOnce(&mut VirtualScreen) -> R) -> R {
    let screen_index = active_screen_index();
    with_screen_mut(screen_index, f).expect("active screen index out of bounds")
}

pub(super) fn sync_cursor_of(screen: &VirtualScreen) {
    cursor::sync_hardware_cursor(screen);
}

pub(super) fn sync_screen_state(screen: &mut VirtualScreen) {
    screen.used_lines = screen.used_lines.min(SCROLLBACK_LINES);
    screen.viewport = screen
        .viewport
        .min(screen.used_lines.saturating_sub(VGA_HEIGHT));
}

pub(super) fn is_screen_active(screen: &VirtualScreen) -> bool {
    screen.active
}

pub(super) fn set_active(screen_index: usize) {
    with_screen_mut(screen_index, |screen| {
        screen.active = true;
        render_screen(screen);
    });
    ACTIVE_SCREEN_IDX.store(screen_index, Ordering::Relaxed);
}

pub(super) fn init() {
    VIRTUAL_SCREENS.for_each_mut(|screen| {
        for line in 0..SCROLLBACK_LINES {
            clear_buffer_line(screen, line);
        }
        screen.accepts_input = false;
    });

    set_active(0);
}
