use core::arch::asm;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    flags: u8,
    offset_high: u16,
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u32,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    zero: 0,
    flags: 0,
    offset_high: 0,
}; 256];

pub fn init_idt() {
    let idt_ptr = IdtPointer {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: unsafe { &IDT as *const _ as u32 },
    };

    unsafe {
        asm!(
            "lidt [{}]",
            in(reg) &idt_ptr,
            options(nostack, preserves_flags)
        );
    }
}

pub unsafe fn register_interrupt_handler(index: u8, handler: extern "C" fn()) {
    let handler_addr = handler as u32;
    IDT[index as usize] = IdtEntry {
        offset_low: handler_addr as u16,
        selector: 0x08, // Kernel code segment
        zero: 0,
        flags: 0x8E, // Present, DPL=0, 32-bit interrupt gate
        offset_high: (handler_addr >> 16) as u16,
    };
}
