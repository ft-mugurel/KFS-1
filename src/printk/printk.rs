use core::fmt;
use core::sync::atomic::Ordering;

use crate::vga::text_mod::out;
use crate::{DEFAULT_LOG_SCREEN, LOG_LEVEL};

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
    Default = 8,
    Cont = 9,
}

#[allow(non_camel_case_types)]
pub type KERNEL_LOG_LEVEL = KernelLogLevel;

const fn normalize_level(level: KernelLogLevel) -> u8 {
    match level {
        KernelLogLevel::Default => KernelLogLevel::Info as u8,
        KernelLogLevel::Cont => KernelLogLevel::Debug as u8,
        _ => level as u8,
    }
}

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
        8 => KernelLogLevel::Default,
        9 => KernelLogLevel::Cont,
        _ => KernelLogLevel::Info,
    }
}

fn color_for_level(level: KernelLogLevel) -> out::ColorCode {
    match level {
        KernelLogLevel::Emerg | KernelLogLevel::Alert | KernelLogLevel::Crit => {
            out::ColorCode::new(out::Color::LightRed, out::Color::Black)
        }
        KernelLogLevel::Err => out::ColorCode::new(out::Color::Red, out::Color::Black),
        KernelLogLevel::Warning => out::ColorCode::new(out::Color::Yellow, out::Color::Black),
        KernelLogLevel::Notice => out::ColorCode::new(out::Color::LightBlue, out::Color::Black),
        KernelLogLevel::Info => out::ColorCode::new(out::Color::White, out::Color::Black),
        KernelLogLevel::Debug => out::ColorCode::new(out::Color::LightGray, out::Color::Black),
        _ => out::ColorCode::new(out::Color::White, out::Color::Black),
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
        KernelLogLevel::Default => "DEFAULT",
        KernelLogLevel::Cont => "CONT",
    }
}

pub fn set_log_level(level: KernelLogLevel) {
    LOG_LEVEL.store(normalize_level(level), Ordering::Relaxed);
}

pub fn log_level() -> KernelLogLevel {
    level_from_u8(LOG_LEVEL.load(Ordering::Relaxed))
}

fn is_enabled(level: KernelLogLevel) -> bool {
    let current = LOG_LEVEL.load(Ordering::Relaxed);
    let wanted = normalize_level(level);
    wanted <= current
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
        DEFAULT_LOG_SCREEN.load(Ordering::Relaxed),
        color_for_level(level),
    );
    out::write_fmt_on(
        DEFAULT_LOG_SCREEN.load(Ordering::Relaxed),
        format_args!("[{}] ", level_tag(level)),
    );
    out::write_fmt_on(DEFAULT_LOG_SCREEN.load(Ordering::Relaxed), args);
}

pub fn printk_to_screen(screen_index: usize, args: fmt::Arguments<'_>) {
    printk_level_to_screen(screen_index, KernelLogLevel::Info, args);
}

pub fn printk_to_default(args: fmt::Arguments<'_>) {
    printk_level_to_default(KernelLogLevel::Info, args);
}
