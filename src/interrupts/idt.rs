use core::arch::asm;

// This creates and IDT Struct and make it like __attribute__((packed)) in C
#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    flags: u8,
    offset_high: u16,
}

// This creates an IDT Pointer Struct and make it like __attribute__((packed)) in C
#[repr(C, packed)]
struct IdtPointer {
    limit: u16, // Size of the IDT - 1
    base: u32,  // Address of the first IDT entry
}

// Creates an IDT Struct array with 256 entries and set them to zero
static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    zero: 0,
    flags: 0,
    offset_high: 0,
}; 256];

pub fn init_idt() {
    // Initialize the IDTPointer first entry the size second the address of the first IDT entry
    let idt_ptr = IdtPointer {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: &raw const IDT as *const _ as u32,
    };

    // Load the IDT using the lidt (Load Interrupt Descriptor Table) instruction
    unsafe {
        asm!( "lidt [{}]", in(reg) &idt_ptr, options(nostack, preserves_flags));
    }
}

pub(crate) unsafe fn register_interrupt_handler(index: u8, handler: unsafe extern "C" fn()) {
    let handler_addr = handler as u32;
    IDT[index as usize] = IdtEntry {
        offset_low: handler_addr as u16,
        selector: 0x08, // Kernel code segment
        zero: 0,
        flags: 0x8E, // Present, DPL=0, 32-bit interrupt gate
        offset_high: (handler_addr >> 16) as u16,
    };
}
