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

const GDT_LIMIT_BYTES: u32 = 10 * 1024 * 1024;
const GDT_LIMIT_FIELD: u32 = (GDT_LIMIT_BYTES >> 12) - 1;
const GDT_GRANULARITY: u8 = 0xC0 | ((GDT_LIMIT_FIELD >> 16) as u8 & 0x0F);

const fn make_entry(access: u8) -> GdtEntry {
    GdtEntry {
        limit_low: GDT_LIMIT_FIELD as u16,
        base_low: 0,
        base_middle: 0,
        access,
        granularity: GDT_GRANULARITY,
        base_high: 0,
    }
}

#[unsafe(link_section = ".gdt")]
#[used]
static GDT: [GdtEntry; 7] = [
    GdtEntry {
        limit_low: 0,
        base_low: 0,
        base_middle: 0,
        access: 0,
        granularity: 0,
        base_high: 0,
    }, // Null segment
    make_entry(0x9A), // Kernel code
    make_entry(0x92), // Kernel data
    make_entry(0x96), // Kernel stack (expand-down data segment)
    make_entry(0xFA), // User code
    make_entry(0xF2), // User data
    make_entry(0xF6), // User stack (expand-down data segment)
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

            // Switch to code segment (0x08) with a far return
            "push 0x08",
            "lea eax, [2f]",
            "push eax",
            "retf",
            "2:",
            out("eax") _,
        );
    }
}
