use core::fmt;
use core::sync::atomic::{AtomicU8, Ordering};

use crate::startup_config;
use crate::vga::text_mod::out;

pub static LOG_LEVEL: AtomicU8 = AtomicU8::new(KernelLogLevel::Info as u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum KernelLogLevel {
    Emerg = 0,
    Alert = 1,
    Crit = 2,
    Err = 3,
    Warning = 4,
    Notice = 5,
    Info = 6,
    Debug = 7,
}

#[allow(non_camel_case_types)]
pub type KERNEL_LOG_LEVEL = KernelLogLevel;

fn level_from_u8(level: u8) -> KernelLogLevel {
    match level {
        0 => KernelLogLevel::Emerg,
        1 => KernelLogLevel::Alert,
        2 => KernelLogLevel::Crit,
        3 => KernelLogLevel::Err,
        4 => KernelLogLevel::Warning,
        5 => KernelLogLevel::Notice,
        6 => KernelLogLevel::Info,
        7 => KernelLogLevel::Debug,
        _ => KernelLogLevel::Info,
    }
}

fn color_for_level(level: KernelLogLevel) -> out::ColorCode {
    match level {
        KernelLogLevel::Emerg => out::ColorCode::new(out::Color::White, out::Color::Red),
        KernelLogLevel::Alert | KernelLogLevel::Crit => {
            out::ColorCode::new(out::Color::LightRed, out::Color::Black)
        }
        KernelLogLevel::Err => out::ColorCode::new(out::Color::Red, out::Color::Black),
        KernelLogLevel::Warning => out::ColorCode::new(out::Color::Yellow, out::Color::Black),
        KernelLogLevel::Notice => out::ColorCode::new(out::Color::LightBlue, out::Color::Black),
        KernelLogLevel::Info => out::ColorCode::new(out::Color::White, out::Color::Black),
        KernelLogLevel::Debug => out::ColorCode::new(out::Color::DarkGray, out::Color::Black),
    }
}

fn level_tag(level: KernelLogLevel) -> &'static str {
    match level {
        KernelLogLevel::Emerg => "EMERG",
        KernelLogLevel::Alert => "ALERT",
        KernelLogLevel::Crit => "CRIT",
        KernelLogLevel::Err => "ERR",
        KernelLogLevel::Warning => "WARN",
        KernelLogLevel::Notice => "NOTICE",
        KernelLogLevel::Info => "INFO",
        KernelLogLevel::Debug => "DEBUG",
    }
}

pub fn set_log_level(level: KernelLogLevel) {
    LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

pub fn log_level() -> KernelLogLevel {
    level_from_u8(LOG_LEVEL.load(Ordering::Relaxed))
}

fn is_enabled(level: KernelLogLevel) -> bool {
    level as u8 <= LOG_LEVEL.load(Ordering::Relaxed)
}

pub fn printk_level_to_screen(
    screen_index: usize,
    level: KernelLogLevel,
    args: fmt::Arguments<'_>,
) {
    if !is_enabled(level) {
        return;
    }

    out::change_color_on(screen_index, color_for_level(level));
    out::write_fmt_on(screen_index, format_args!("[{}] ", level_tag(level)));
    out::write_fmt_on(screen_index, args);
}

pub fn printk_level_to_default(level: KernelLogLevel, args: fmt::Arguments<'_>) {
    if !is_enabled(level) {
        return;
    }

    out::change_color_on(
        startup_config::logging::DEFAULT_LOG_SCREEN,
        color_for_level(level),
    );
    out::write_fmt_on(
        startup_config::logging::DEFAULT_LOG_SCREEN,
        format_args!("[{}] ", level_tag(level)),
    );
    out::write_fmt_on(startup_config::logging::DEFAULT_LOG_SCREEN, args);
}

pub fn printk_to_screen(screen_index: usize, args: fmt::Arguments<'_>) {
    printk_level_to_screen(screen_index, KernelLogLevel::Info, args);
}

pub fn printk_to_default(args: fmt::Arguments<'_>) {
    printk_level_to_default(KernelLogLevel::Info, args);
}

pub fn printk_to_debug(args: fmt::Arguments<'_>) {
    let screen_index = if is_enabled(KernelLogLevel::Debug) {
        startup_config::logging::DEFAULT_LOG_SCREEN
    } else {
        startup_config::logging::DEFAULT_DEBUG_LOG_SCREEN
    };
    out::change_color_on(screen_index, color_for_level(KernelLogLevel::Debug));
    out::write_fmt_on(
        screen_index,
        format_args!("[{}] ", level_tag(KernelLogLevel::Debug)),
    );
    out::write_fmt_on(screen_index, args);
}
