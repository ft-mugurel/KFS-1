#![no_std]
#![no_main]

pub mod gdt;
pub mod debug;
pub mod interrupts;
pub mod paging;
pub mod printk;
pub mod shell;
pub mod startup_config;
pub mod vga;
pub mod x86;

use core::panic::PanicInfo;

use gdt::gdt::load_gdt;

use interrupts::exceptions::init_exceptions;
use interrupts::idt::init_idt;
use interrupts::keyboard::init::init_keyboard;
use interrupts::pic::init_pic;
use interrupts::utils::enable_interrupts;
use paging::init::init_paging;
use shell::init::init_shell;
use vga::text_mod::out::init_virtual_screens;
use x86::{disable_interrupts, hlt_loop};

use crate::{
    startup_config::logging::DEFAULT_LOG_SCREEN,
    vga::text_mod::out::{set_screen_accepts_input, switch_screen},
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    pr_emerg!("KERNEL PANIC\n");
    pr_emerg!("{}\n", info);
    disable_interrupts();
    switch_screen(DEFAULT_LOG_SCREEN);
    hlt_loop();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(multiboot_magic: u32, multiboot_info_addr: u32) -> ! {
    disable_interrupts();
    init_virtual_screens();
    pr_info!(
        "startup config: vga={}x{}, virtual_screens={}, scrollback={}\n",
        startup_config::vga::WIDTH,
        startup_config::vga::HEIGHT,
        startup_config::vga::VIRTUAL_SCREENS,
        startup_config::vga::SCROLLBACK_LINES,
    );
    load_gdt();
    init_idt();
    init_exceptions();
    unsafe { init_pic() };
    init_paging(multiboot_magic, multiboot_info_addr);
    init_keyboard();
    pr_info!("Startup complete, enabling interrupts and shell\n",);
    init_shell();
    set_screen_accepts_input(startup_config::shell::SCREEN_INDEX, true);
    enable_interrupts();
    hlt_loop();
}
