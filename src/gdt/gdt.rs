use core::arch::asm;
use core::mem::size_of;

#[repr(C, packed)] // This struct ensures correct memory alignment and packing
#[derive(Copy, Clone)]
pub struct GdtEntry {
    pub limit_low: u16,      // Lower part of the segment limit
    pub base_low: u16,       // Lower part of the segment base address
    pub base_middle: u8,     // Middle part of the segment base address
    pub access: u8,          // Access byte (defines properties like read/write)
    pub granularity: u8,     // Granularity byte (defines 16-bit vs 32-bit)
    pub base_high: u8,       // Higher part of the segment base address
}

#[repr(C, packed)] // This struct is used to point to the GDT in memory
pub struct GdtPointer {
    pub limit: u16,  // Size of the GDT - 1
    pub base: u32,   // Base address of the GDT
}

static mut GDT: [GdtEntry; 3] = [
    GdtEntry { 
        limit_low: 0, 
        base_low: 0, 
        base_middle: 0, 
        access: 0, 
        granularity: 0, 
        base_high: 0, 
    }, // Null Segment (GDT[0])
    GdtEntry { 
        limit_low: 0xFFFF, 
        base_low: 0, 
        base_middle: 0, 
        access: 0x9A, // Access byte (present, ring 0, code segment)
        granularity: 0xCF, // Granularity byte (32-bit)
        base_high: 0, 
    }, // Code Segment (GDT[1])
    GdtEntry { 
        limit_low: 0xFFFF, 
        base_low: 0, 
        base_middle: 0, 
        access: 0x92, // Access byte (present, ring 0, data segment)
        granularity: 0xCF, 
        base_high: 0, 
    }, // Data Segment (GDT[2])
];

pub fn load_gdt() {
    // We use an unsafe block to access the mutable static GDT

    let gdt_ptr =  GdtPointer {
        limit: (size_of::<[GdtEntry; 7]>() - 1) as u16,
        base: &raw const GDT as *const _ as usize as u32,
    };

    unsafe {
        // Load the GDT using the LGDT instruction
        asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(nostack, preserves_flags)
        );

        // Reload the segment registers with the correct values
        asm!(
            // Set up data segment registers to 0x10 (data segment selector)
            "mov ax, 0x10",   // Load the data segment selector (2nd entry) into ax
            "mov ds, ax",     // Move the value of ax into the data segment register
            "mov es, ax",     // Move the value of ax into the extra segment register
            "mov fs, ax",     // Move the value of ax into the fs segment register
            "mov gs, ax",     // Move the value of ax into the gs segment register
            "mov ss, ax",     // Move the value of ax into the stack segment register

            // Switch to code segment (0x08) and jump to the next instruction
            "mov ax, 0x08",   // Load the code segment selector (1st entry) into ax
            "mov cs, ax",     // Set the code segment register to 0x08 (the first entry)
            "jmp 2f",         // Jump to label '2'

            // Label '2:', indicating where the jump will land
            "2:",             

            options(nostack)
        );
    }
}
