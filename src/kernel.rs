#![no_std]
#![no_main]

pub mod gdt;
pub mod interrupts;
pub mod printk;
pub mod shell;
pub mod vga;
pub mod x86;

use core::panic::PanicInfo;

use gdt::gdt::load_gdt;

use interrupts::idt::init_idt;
use interrupts::keyboard::init::init_keyboard;
use interrupts::pic::init_pic;
use interrupts::utils::enable_interrupts;
use vga::text_mod::out::init_virtual_screens;

use crate::vga::text_mod::out::set_screen_accepts_input;

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
    pr_info!("Kernel initialized {}!\n", 4 + 2);
    pr_alert!("This is the first virtual screen\n");
    pr_info!("Only this one accepts input and has the debug shell.\n");
    pr_emerg!("F1-F6 to switch screens.\n");
    set_screen_accepts_input(0, true);
    printk_on!(1, "This is the second virtual screen\n");
    printk_on!(2, "This is another virtual screen\n");
    printk_on!(3, "This is F4, in case you were wondering\n");
    printk_on!(4, "This is F5\n");
    printk_on!(5, "This is F6\n");
    loop {}
}
