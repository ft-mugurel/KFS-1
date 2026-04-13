use core::cell::UnsafeCell;
use core::fmt;
use core::ptr;
use core::str;

use crate::interrupts::keyboard::character_map::keycode_to_char;
use crate::interrupts::keyboard::keycode::{KeyCode, KeyEvent, Modifiers};
use crate::interrupts::utils::{request_reboot, request_shutdown};
use crate::paging::{kernel_heap, page_table, physical, vmem};
use crate::printk::{set_log_level, KernelLogLevel};
use crate::startup_config;
use crate::vga::text_mod::out::{
    self, active_screen_accepts_input, change_color, clear, print_char_on, print_on,
    set_cursor_movement_on, switch_screen, Color, ColorCode,
};
use crate::vga::text_mod::screen;

const PROMPT: &str = "mysh > ";
const MAX_INPUT_LEN: usize = startup_config::shell::MAX_INPUT_LEN;
const SCREEN_INDEX: usize = startup_config::shell::SCREEN_INDEX;
const MEMDUMP_DEFAULT_LEN: usize = 128;
const MEMDUMP_MAX_LEN: usize = 512;

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

#[inline]
fn print(s: &str) {
    print_on(SCREEN_INDEX, s);
}

#[inline]
fn print_char(c: char) {
    print_char_on(SCREEN_INDEX, c);
}

#[inline]
fn print_fmt(args: fmt::Arguments<'_>) {
    out::write_fmt_on(SCREEN_INDEX, args);
}

pub fn init_shell() {
    with_shell_state_mut(|state| {
        if state.initialized {
            return;
        }

        state.initialized = true;
        print("This is the default screen for the shell\n");
        print("Use F1-F6 / Shift+<Left/Right Arrow> to switch screens.\n");
        print(PROMPT);
        state.rendered_len = PROMPT.len();
        set_cursor_movement_on(SCREEN_INDEX, screen::CursorMovement::Horizontal);
    });
}

pub fn handle_shell_key_event(event: KeyEvent, modifiers: Modifiers) -> bool {
    if !out::is_screen_active(SCREEN_INDEX) || !active_screen_accepts_input() {
        return false;
    }

    match event.key {
        KeyCode::Home => {
            with_shell_state_mut(|state| {
                state.idx = 0;
            });
            redraw_input_line();
            true
        }
        KeyCode::End => {
            with_shell_state_mut(|state| {
                state.idx = state.len;
            });
            redraw_input_line();
            true
        }
        KeyCode::ArrowLeft => {
            if modifiers.shift() {
                out::switch_to_previous_screen();
            } else if with_shell_state_mut(|state| state.idx_left()) {
                redraw_input_line();
            }
            true
        }
        KeyCode::ArrowRight => {
            if modifiers.shift() {
                out::switch_to_next_screen();
            } else if with_shell_state_mut(|state| state.idx_right()) {
                redraw_input_line();
            }
            true
        }
        KeyCode::Enter => {
            print_char('\n');
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
                    print(
                        "\nhelp clear echo shutdown reboot screen loglevel color memstat memdebug memdump pte memtest\n",
                    );
                    state.clear_input();
                    redraw_input_line();
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
            } else {
                if event.key == KeyCode::C && modifiers.ctrl() {
                    print_char('\n');
                    with_shell_state_mut(|state| {
                        state.clear_input();
                    });
                    redraw_input_line();
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

        print_char('\r');
        print(PROMPT);

        if let Ok(input_str) = str::from_utf8(&state.input[..state.len]) {
            print(input_str);
        }

        if old_rendered_len > new_rendered_len {
            for _ in 0..(old_rendered_len - new_rendered_len) {
                print_char(' ');
            }
        }

        let cursor_offset = PROMPT.len() + state.idx;
        let new_cursor_x = (cursor_offset % screen::VGA_WIDTH) as u16;
        let new_cursor_y = cursor_y + (cursor_offset / screen::VGA_WIDTH) as u16;
        out::set_cursor_position_on(SCREEN_INDEX, new_cursor_x, new_cursor_y);
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
        print(PROMPT);
        with_shell_state_mut(|state| {
            state.idx = 0;
            state.rendered_len = PROMPT.len();
        });
        return;
    }

    run_command(line);

    if out::is_screen_active(SCREEN_INDEX) {
        print(PROMPT);
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
            print(
                "Commands: help clear echo shutdown reboot screen loglevel color memstat memdebug memdump pte memtest\n",
            );
            print("screen <1-6>\n");
            print("loglevel <emerg|alert|crit|err|warn|notice|info|debug>\n");
            print("color <white|gray|red|green|blue|yellow|cyan|magenta>\n");
            print("memstat\n");
            print("memdebug\n");
            print("memdump <addr> [len<=512]\n");
            print("pte <addr>\n");
            print("memtest [physical,vmem,heap,page,all]\n");
        }
        "clear" => clear(SCREEN_INDEX),
        "echo" => {
            let rest = line[command.len()..].trim_start();
            print(rest);
            print_char('\n');
        }
        "reboot" => request_reboot(),
        "shutdown" => request_shutdown(),
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
        "memstat" => command_memstat(),
        "memdebug" => command_memdebug(),
        "memdump" => {
            let Some(addr_str) = parts.next() else {
                print("usage: memdump <addr> [len<=512]\n");
                return;
            };

            let Some(addr) = parse_u32(addr_str) else {
                print("invalid address\n");
                return;
            };

            let len = if let Some(len_str) = parts.next() {
                let Some(parsed) = parse_usize(len_str) else {
                    print("invalid length\n");
                    return;
                };
                parsed
            } else {
                MEMDUMP_DEFAULT_LEN
            };

            if len == 0 || len > MEMDUMP_MAX_LEN {
                print("length must be in range 1..=512\n");
                return;
            }

            dump_virtual_memory(addr, len);
        }
        "pte" => {
            let Some(addr_str) = parts.next() else {
                print("usage: pte <addr>\n");
                return;
            };

            let Some(addr) = parse_u32(addr_str) else {
                print("invalid address\n");
                return;
            };

            debug_page_entry(addr);
        }
        "memtest" => {
            let features = line[command.len()..].trim();
            command_memtest(features);
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
        "help", "clear", "echo", "reboot", "shutdown", "screen", "loglevel", "color", "memstat",
        "memdebug", "memdump", "pte", "memtest",
    ];
    let mut matches = commands.iter().filter(|&cmd| cmd.starts_with(_partial));
    let first_match = matches.next()?;
    if matches.next().is_some() {
        print_char('\n');
        print(*first_match);
        matches.for_each(|&cmd| print_fmt(format_args!(" {}", cmd)));
        print_char('\n');
        redraw_input_line();
        None
    } else {
        Some(first_match)
    }
}

fn parse_u32(input: &str) -> Option<u32> {
    if let Some(hex) = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else {
        input.parse::<u32>().ok()
    }
}

fn parse_usize(input: &str) -> Option<usize> {
    if let Some(hex) = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
    {
        usize::from_str_radix(hex, 16).ok()
    } else {
        input.parse::<usize>().ok()
    }
}

fn command_memstat() {
    let total_pages = physical::total_physical_pages();
    let free_pages = physical::free_physical_pages();
    let used_pages = total_pages.saturating_sub(free_pages);
    let page_size = physical::physical_page_size();
    let total_phys_kib = total_pages.saturating_mul(page_size) / 1024;
    let free_phys_kib = free_pages.saturating_mul(page_size) / 1024;

    let vstats = vmem::debug_stats();
    let hstats = kernel_heap::debug_stats();

    print_fmt(format_args!(
        "physical: total_pages={} free_pages={} used_pages={} total_kib={} free_kib={}\n",
        total_pages, free_pages, used_pages, total_phys_kib, free_phys_kib
    ));
    print_fmt(format_args!(
        "vmem: range=[{:#010x}, {:#010x}) total_kib={} free_kib={} allocs={} alloc_bytes={} free_ranges={}\n",
        vstats.range_start,
        vstats.range_end,
        (vstats.total_bytes as usize) / 1024,
        (vstats.free_bytes as usize) / 1024,
        vstats.alloc_count,
        vstats.alloc_bytes,
        vstats.free_ranges
    ));
    print_fmt(format_args!(
        "kheap: ready={} chunks={} chunk_bytes={} free_blocks={} free_bytes={} used_blocks={} used_req_bytes={}\n",
        hstats.ready,
        hstats.chunk_count,
        hstats.chunk_bytes,
        hstats.free_block_count,
        hstats.free_bytes,
        hstats.used_block_count,
        hstats.used_requested_bytes
    ));
}

fn command_memdebug() {
    command_memstat();

    print("vmem active allocations:\n");
    let mut alloc_count = 0usize;
    vmem::debug_for_each_alloc(|base, size, pages| {
        alloc_count += 1;
        print_fmt(format_args!(
            "  alloc#{:02} base={:#010x} size={} pages={}\n",
            alloc_count, base, size, pages
        ));
    });
    if alloc_count == 0 {
        print("  (none)\n");
    }

    print("vmem free ranges:\n");
    let mut free_count = 0usize;
    vmem::debug_for_each_free_range(|base, size| {
        free_count += 1;
        let end = base.saturating_add(size);
        print_fmt(format_args!(
            "  range#{:02} [{:#010x}, {:#010x}) size={}\n",
            free_count, base, end, size
        ));
    });
    if free_count == 0 {
        print("  (none)\n");
    }
}

fn debug_page_entry(addr: u32) {
    let page_base = addr & 0xFFFF_F000;
    match page_table::get_page(page_base) {
        Some(entry) => {
            let phys = entry & 0xFFFF_F000;
            let flags = entry & 0x0000_0FFF;
            print_fmt(format_args!(
                "pte: va={:#010x} page={:#010x} entry={:#010x} pa={:#010x} flags={:#05x}\n",
                addr, page_base, entry, phys, flags
            ));
            print_fmt(format_args!(
                "  present={} writable={} user={} huge={}\n",
                (entry & page_table::PAGE_PRESENT) != 0,
                (entry & page_table::PAGE_WRITABLE) != 0,
                (entry & page_table::PAGE_USER) != 0,
                (entry & page_table::PAGE_PAGE_SIZE_4MB) != 0
            ));
        }
        None => {
            print_fmt(format_args!(
                "pte: va={:#010x} page={:#010x} not mapped\n",
                addr, page_base
            ));
        }
    }
}

fn dump_virtual_memory(start_addr: u32, len: usize) {
    let end_addr = match (start_addr as usize).checked_add(len) {
        Some(v) if v <= u32::MAX as usize => v as u32,
        _ => {
            print("address range overflow\n");
            return;
        }
    };

    print_fmt(format_args!(
        "memdump: [{:#010x}, {:#010x}) len={}\n",
        start_addr, end_addr, len
    ));

    let mut offset = 0usize;
    while offset < len {
        let line_addr = start_addr.wrapping_add(offset as u32);
        print_fmt(format_args!("{:#010x}: ", line_addr));

        let mut ascii = [b'.'; 16];
        for i in 0usize..16 {
            let pos = offset + i;
            if pos >= len {
                print("   ");
                continue;
            }

            let byte_addr = start_addr.wrapping_add(pos as u32);
            let page_base = byte_addr & 0xFFFF_F000;
            if page_table::get_page(page_base).is_none() {
                print("?? ");
                ascii[i] = b'?';
                continue;
            }

            let value = unsafe { ptr::read_volatile(byte_addr as *const u8) };
            print_fmt(format_args!("{:02x} ", value));
            ascii[i] = if value.is_ascii_graphic() || value == b' ' {
                value
            } else {
                b'.'
            };
        }

        print(" |");
        for i in 0usize..16 {
            let pos = offset + i;
            if pos >= len {
                break;
            }
            print_char(ascii[i] as char);
        }
        print("|\n");

        offset = offset.saturating_add(16);
    }
}

fn command_memtest(features: &str) {
    let mut run_physical = false;
    let mut run_vmem = false;
    let mut run_heap = false;
    let mut run_page = false;

    if features.is_empty() || features == "all" {
        run_physical = true;
        run_vmem = true;
        run_heap = true;
        run_page = true;
    } else {
        for token in features.split(|c: char| c == ',' || c.is_ascii_whitespace()) {
            if token.is_empty() {
                continue;
            }

            match token {
                "physical" => run_physical = true,
                "vmem" => run_vmem = true,
                "heap" => run_heap = true,
                "page" => run_page = true,
                "all" => {
                    run_physical = true;
                    run_vmem = true;
                    run_heap = true;
                    run_page = true;
                }
                _ => {
                    print("usage: memtest [physical,vmem,heap,page,all]\n");
                    print("unknown feature: ");
                    print(token);
                    print("\n");
                    return;
                }
            }
        }
    }

    print("memtest: running selected tests\n");

    let mut total = 0usize;
    let mut passed = 0usize;

    if run_physical {
        total += 1;
        if memtest_physical_roundtrip() {
            passed += 1;
            print("  [PASS] physical\n");
        } else {
            print("  [FAIL] physical\n");
        }
    }

    if run_vmem {
        total += 1;
        if memtest_vmem_roundtrip() {
            passed += 1;
            print("  [PASS] vmem\n");
        } else {
            print("  [FAIL] vmem\n");
        }
    }

    if run_heap {
        total += 1;
        if memtest_heap_roundtrip() {
            passed += 1;
            print("  [PASS] heap\n");
        } else {
            print("  [FAIL] heap\n");
        }
    }

    if run_page {
        total += 1;
        if memtest_page_roundtrip() {
            passed += 1;
            print("  [PASS] page\n");
        } else {
            print("  [FAIL] page\n");
        }
    }

    print_fmt(format_args!(
        "memtest summary: passed={}/{} failed={}\n",
        passed,
        total,
        total.saturating_sub(passed)
    ));
}

fn memtest_physical_roundtrip() -> bool {
    let free_before = physical::free_physical_pages();
    let Some(frame) = physical::alloc_physical_page() else {
        return false;
    };

    let free_after_alloc = physical::free_physical_pages();
    if free_after_alloc.saturating_add(1) != free_before {
        let _ = physical::free_physical_page(frame);
        return false;
    }

    if !physical::free_physical_page(frame) {
        return false;
    }

    physical::free_physical_pages() == free_before
}

fn memtest_vmem_roundtrip() -> bool {
    let Some(ptr) = vmem::vmalloc(4096) else {
        return false;
    };

    if vmem::vsize(ptr as *const u8) != Some(4096) {
        let _ = vmem::vfree(ptr);
        return false;
    }

    vmem::vfree(ptr)
}

fn memtest_heap_roundtrip() -> bool {
    let Some(ptr) = kernel_heap::kmalloc(128) else {
        return false;
    };

    if kernel_heap::ksize(ptr as *const u8) != Some(128) {
        let _ = kernel_heap::kfree(ptr);
        return false;
    }

    kernel_heap::kfree(ptr)
}

fn memtest_page_roundtrip() -> bool {
    const TEST_USER_VA: u32 = 0x0800_0000;

    let Some(frame) = physical::alloc_physical_page() else {
        return false;
    };

    let map_ok = page_table::map_page(
        TEST_USER_VA,
        frame,
        page_table::PAGE_WRITABLE | page_table::PAGE_USER,
    )
    .is_ok();
    if !map_ok {
        let _ = physical::free_physical_page(frame);
        return false;
    }

    let mapped =
        matches!(page_table::get_page(TEST_USER_VA), Some(entry) if (entry & 0xFFFF_F000) == frame);

    let _ = page_table::unmap_page(TEST_USER_VA);
    let _ = physical::free_physical_page(frame);

    mapped
}
