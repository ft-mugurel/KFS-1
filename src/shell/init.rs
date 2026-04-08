use core::cell::UnsafeCell;
use core::str;

use crate::interrupts::keyboard::character_map::keycode_to_char;
use crate::interrupts::keyboard::keycode::{KeyCode, KeyEvent, Modifiers};
use crate::interrupts::utils::{request_reboot, request_shutdown};
use crate::printk::{set_log_level, KernelLogLevel};
use crate::vga::text_mod::out::{
    self, active_screen_accepts_input, change_color, clear, newline_on, print, print_char_on,
    print_on, set_cursor_movement_on, switch_screen, Color, ColorCode,
};
use crate::vga::text_mod::screen;
use crate::SHELL_SCREEN_INDEX;

const PROMPT: &str = "debug> ";
const MAX_INPUT_LEN: usize = 128;

struct ShellState {
    input: [u8; MAX_INPUT_LEN],
    idx: usize,
    len: usize,
    rendered_len: usize,
    initialized: bool,
}

impl ShellState {
    const fn new() -> Self {
        Self {
            input: [0; MAX_INPUT_LEN],
            idx: 0,
            len: 0,
            rendered_len: 0,
            initialized: false,
        }
    }

    fn clear_input(&mut self) {
        self.len = 0;
        self.idx = 0;
        self.rendered_len = 0;
    }

    fn push_char(&mut self, c: char) -> bool {
        if !c.is_ascii() || c.is_ascii_control() || self.len >= MAX_INPUT_LEN {
            return false;
        }
        if self.idx < self.len {
            self.input.copy_within(self.idx..self.len, self.idx + 1);
        }
        self.input[self.idx] = c as u8;
        self.idx += 1;
        self.len += 1;
        true
    }

    fn delete_char(&mut self) -> bool {
        if self.idx == 0 {
            return false;
        }
        self.input.copy_within(self.idx..self.len, self.idx - 1);
        self.idx -= 1;
        self.len -= 1;
        self.input[self.len] = 0;
        true
    }

    fn delete_forward(&mut self) -> bool {
        if self.idx >= self.len {
            return false;
        }

        self.input.copy_within(self.idx + 1..self.len, self.idx);
        self.len -= 1;
        self.input[self.len] = 0;
        true
    }

    fn idx_left(&mut self) -> bool {
        if self.idx == 0 {
            return false;
        }
        self.idx -= 1;
        true
    }

    fn idx_right(&mut self) -> bool {
        if self.idx >= self.len {
            return false;
        }
        self.idx += 1;
        true
    }
}

struct ShellStateCell(UnsafeCell<ShellState>);

unsafe impl Sync for ShellStateCell {}

static SHELL_STATE: ShellStateCell = ShellStateCell(UnsafeCell::new(ShellState::new()));

fn with_shell_state_mut<R>(f: impl FnOnce(&mut ShellState) -> R) -> R {
    let state = unsafe { &mut *SHELL_STATE.0.get() };
    f(state)
}

pub fn init_shell() {
    with_shell_state_mut(|state| {
        if state.initialized {
            return;
        }

        state.initialized = true;
        print_on(SHELL_SCREEN_INDEX, PROMPT);
        state.rendered_len = PROMPT.len();
        set_cursor_movement_on(SHELL_SCREEN_INDEX, screen::CursorMovement::Horizontal);
    });
}

pub fn handle_shell_key_event(event: KeyEvent, modifiers: Modifiers) -> bool {
    if !out::is_screen_active(SHELL_SCREEN_INDEX) || !active_screen_accepts_input() {
        return false;
    }

    match event.key {
        KeyCode::ArrowLeft => {
            if with_shell_state_mut(|state| state.idx_left()) {
                redraw_input_line();
            }
            true
        }
        KeyCode::ArrowRight => {
            if with_shell_state_mut(|state| state.idx_right()) {
                redraw_input_line();
            }
            true
        }
        KeyCode::Enter => {
            newline_on(SHELL_SCREEN_INDEX);
            run_command_line();
            true
        }
        KeyCode::Backspace => {
            let removed = with_shell_state_mut(|state| state.delete_char());
            if removed {
                redraw_input_line();
            }
            true
        }
        KeyCode::Delete => {
            let removed = with_shell_state_mut(|state| state.delete_forward());
            if removed {
                redraw_input_line();
            }
            true
        }
        KeyCode::Tab => {
            with_shell_state_mut(|state| {
                if state.idx == 0 || state.input[state.idx - 1] == b' ' {
                    newline_on(SHELL_SCREEN_INDEX);
                    print_on(
                        SHELL_SCREEN_INDEX,
                        "help clear echo shutdown screen loglevel color\n",
                    );
                    print_on(SHELL_SCREEN_INDEX, PROMPT);
                    state.idx = 0;
                    state.len = 0;
                    state.rendered_len = PROMPT.len();
                    return;
                }
                let start = state.input[..state.idx]
                    .iter()
                    .rposition(|&b| b == b' ')
                    .map_or(0, |pos| pos + 1);
                let partial = str::from_utf8(&state.input[start..state.idx]).unwrap_or("");
                if let Some(completion) = complete(partial) {
                    for c in completion[partial.len()..].chars() {
                        state.push_char(c);
                    }
                    redraw_input_line();
                }
            });
            true
        }
        _ => {
            if !modifiers.has_text_blocking_modifier() {
                if let Some(ch) = keycode_to_char(event.key, modifiers) {
                    let _ = try_insert_char(ch);
                    return true;
                }
            }
            false
        }
    }
}

fn try_insert_char(c: char) -> bool {
    let inserted = with_shell_state_mut(|state| state.push_char(c));
    if inserted {
        redraw_input_line();
    }
    inserted
}

fn redraw_input_line() {
    let (cursor_x, cursor_y) = out::active_cursor_position();

    with_shell_state_mut(|state| {
        let old_rendered_len = state.rendered_len;
        let new_rendered_len = PROMPT.len() + state.len;

        print_char_on(SHELL_SCREEN_INDEX, '\r');
        print_on(SHELL_SCREEN_INDEX, PROMPT);

        if let Ok(input_str) = str::from_utf8(&state.input[..state.len]) {
            print_on(SHELL_SCREEN_INDEX, input_str);
        }

        if old_rendered_len > new_rendered_len {
            for _ in 0..(old_rendered_len - new_rendered_len) {
                print_char_on(SHELL_SCREEN_INDEX, ' ');
            }
        }

        let cursor_offset = PROMPT.len() + state.idx;
        let new_cursor_x = (cursor_offset % screen::VGA_WIDTH) as u16;
        let new_cursor_y = cursor_y + (cursor_offset / screen::VGA_WIDTH) as u16;
        out::set_cursor_position_on(SHELL_SCREEN_INDEX, new_cursor_x, new_cursor_y);
        state.rendered_len = new_rendered_len;
        let _ = cursor_x;
    });
}

fn run_command_line() {
    let (len, line_buf) = with_shell_state_mut(|state| {
        let len = state.len;
        let mut buf = [0u8; MAX_INPUT_LEN];
        buf[..len].copy_from_slice(&state.input[..len]);
        state.clear_input();
        (len, buf)
    });

    let line = str::from_utf8(&line_buf[..len]).unwrap_or("");
    let line = line.trim();

    if line.is_empty() {
        print_on(SHELL_SCREEN_INDEX, PROMPT);
        with_shell_state_mut(|state| {
            state.idx = 0;
            state.rendered_len = PROMPT.len();
        });
        return;
    }

    run_command(line);

    if out::is_screen_active(SHELL_SCREEN_INDEX) {
        print_on(SHELL_SCREEN_INDEX, PROMPT);
        with_shell_state_mut(|state| {
            state.idx = 0;
            state.rendered_len = PROMPT.len();
        });
    }
}

fn run_command(line: &str) {
    let mut parts = line.split_whitespace();
    let Some(command) = parts.next() else {
        return;
    };

    match command {
        "help" => {
            print("Commands: help clear echo shutdown screen loglevel color\n");
            print("screen <1-6>\n");
            print("loglevel <emerg|alert|crit|err|warn|notice|info|debug>\n");
            print("color <white|gray|red|green|blue|yellow|cyan|magenta>\n");
        }
        "clear" => clear(),
        "echo" => {
            let rest = line[command.len()..].trim_start();
            print(rest);
            print("\n");
        }
        "reboot" => unsafe { request_reboot() },
        "shutdown" => unsafe { request_shutdown() },
        "screen" => {
            let Some(arg) = parts.next() else {
                print("usage: screen <1-6>\n");
                return;
            };

            let Ok(screen) = arg.parse::<usize>() else {
                print("invalid screen index\n");
                return;
            };

            if !(1..=6).contains(&screen) {
                print("screen index must be in range 1-6\n");
                return;
            }

            switch_screen(screen - 1);
        }
        "loglevel" => {
            let Some(arg) = parts.next() else {
                print("usage: loglevel <emerg|alert|crit|err|warn|notice|info|debug>\n");
                return;
            };

            let Some(level) = parse_log_level(arg) else {
                print("invalid log level\n");
                return;
            };

            set_log_level(level);
            print("log level updated\n");
        }
        "color" => {
            let Some(arg) = parts.next() else {
                print("usage: color <white|gray|red|green|blue|yellow|cyan|magenta>\n");
                return;
            };

            let Some(color) = parse_color(arg) else {
                print("invalid color\n");
                return;
            };

            change_color(ColorCode::new(color, Color::Black));
            print("shell color updated\n");
        }
        _ => {
            print("unknown command: ");
            print(command);
            print("\n");
        }
    }
}

fn parse_log_level(name: &str) -> Option<KernelLogLevel> {
    match name {
        "emerg" => Some(KernelLogLevel::Emerg),
        "alert" => Some(KernelLogLevel::Alert),
        "crit" => Some(KernelLogLevel::Crit),
        "err" => Some(KernelLogLevel::Err),
        "warn" | "warning" => Some(KernelLogLevel::Warning),
        "notice" => Some(KernelLogLevel::Notice),
        "info" => Some(KernelLogLevel::Info),
        "debug" => Some(KernelLogLevel::Debug),
        _ => None,
    }
}

fn parse_color(name: &str) -> Option<Color> {
    match name {
        "white" => Some(Color::White),
        "gray" | "lightgray" => Some(Color::LightGray),
        "red" => Some(Color::LightRed),
        "green" => Some(Color::LightGreen),
        "blue" => Some(Color::LightBlue),
        "yellow" => Some(Color::Yellow),
        "cyan" => Some(Color::LightCyan),
        "magenta" => Some(Color::Pink),
        _ => None,
    }
}

fn complete(_partial: &str) -> Option<&'static str> {
    let commands = [
        "help", "clear", "echo", "reboot", "shutdown", "screen", "loglevel", "color",
    ];
    for &cmd in &commands {
        if cmd.starts_with(_partial) {
            return Some(cmd);
        }
    }
    None
}
