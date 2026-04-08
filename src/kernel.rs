#![no_std]
#![no_main]

pub mod gdt;
pub mod interrupts;
pub mod printk;
pub mod shell;
pub mod vga;
pub mod x86;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU8, AtomicUsize};

use gdt::gdt::load_gdt;

use interrupts::idt::init_idt;
use interrupts::keyboard::init::init_keyboard;
use interrupts::pic::init_pic;
use interrupts::utils::enable_interrupts;
use shell::init::init_shell;
use vga::text_mod::out::init_virtual_screens;

use crate::printk::KernelLogLevel;
use crate::vga::text_mod::out::set_screen_accepts_input;

pub const SHELL_SCREEN_INDEX: usize = 0;
pub static LOG_LEVEL: AtomicU8 = AtomicU8::new(KernelLogLevel::Info as u8);
pub static DEFAULT_LOG_SCREEN: AtomicUsize = AtomicUsize::new(1);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    load_gdt();
    init_idt();
    unsafe { init_pic() };
    init_virtual_screens();
    init_keyboard();
    enable_interrupts();
    pr_info!("Kernel initialized!\n",);
    pr_alert!("This should be an alert\n");
    printk_on!(0, "This is the default screen for the shell\n");
    printk_on!(0, "Use F1-F6 to switch screens.\n");
    printk_on!(1, "Default screen for kernel logs\n");
    printk_on!(2, "This is another virtual screen\n");
    printk_on!(3, "This is F4, in case you were wondering\n");
    printk_on!(4, "This is F5\n");
    printk_on!(5, "This is F6\n");
    init_shell();
    set_screen_accepts_input(0, true);
    loop {}
}
