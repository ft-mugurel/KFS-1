use core::arch::asm;
use core::mem::size_of;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct GdtEntry {
    pub limit0: u16,
    pub base0: u16,
    pub base1_flags: u16,
    pub limit1_flags_base2: u16,
}

#[repr(C, packed)]
pub struct GdtPointer {
    pub limit: u16,
    pub base: u32,
}

const GDT_ENTRIES_COUNT: usize = 7;
const GDT_LIMIT_BYTES: u32 = 0xfffff; // 4GiB
const GDT_LIMIT: u32 = (GDT_LIMIT_BYTES >> 12) - 1;

// Mirror Linux arch/x86/include/asm/desc_defs.h flags.
const _DESC_ACCESSED: u16 = 0x0001;
const _DESC_DATA_WRITABLE: u16 = 0x0002;
const _DESC_CODE_READABLE: u16 = 0x0002;
const _DESC_DATA_EXPAND_DOWN: u16 = 0x0004;
const _DESC_CODE_EXECUTABLE: u16 = 0x0008;
const _DESC_S: u16 = 0x0010;
const _DESC_PRESENT: u16 = 0x0080;
const _DESC_DPL3: u16 = 3 << 5;
const _DESC_DB: u16 = 0x4000;
const _DESC_GRANULARITY_4K: u16 = 0x8000;

const DESC_DATA32: u16 = _DESC_S
    | _DESC_PRESENT
    | _DESC_ACCESSED
    | _DESC_DATA_WRITABLE
    | _DESC_GRANULARITY_4K
    | _DESC_DB;
const DESC_CODE32: u16 = _DESC_S
    | _DESC_PRESENT
    | _DESC_ACCESSED
    | _DESC_CODE_READABLE
    | _DESC_CODE_EXECUTABLE
    | _DESC_GRANULARITY_4K
    | _DESC_DB;
const DESC_STACK32: u16 = DESC_DATA32 | _DESC_DATA_EXPAND_DOWN;
const DESC_USER_DATA32: u16 = DESC_DATA32 | _DESC_DPL3;
const DESC_USER_CODE32: u16 = DESC_CODE32 | _DESC_DPL3;
const DESC_USER_STACK32: u16 = DESC_STACK32 | _DESC_DPL3;

const fn make_entry(flags: u16, base: u32, limit: u32) -> GdtEntry {
    // Equivalent to Linux's GDT_ENTRY_INIT(flags, base, limit).
    GdtEntry {
        limit0: ((limit >> 0) & 0xFFFF) as u16,
        base0: ((base >> 0) & 0xFFFF) as u16,
        base1_flags: (((base >> 16) & 0x00FF) as u16) | ((flags & 0x00FF) << 8),
        limit1_flags_base2: (((limit >> 16) & 0x000F) as u16)
            | ((flags >> 8) & 0x00F0)
            | ((((base >> 24) & 0x00FF) as u16) << 8),
    }
}

#[unsafe(link_section = ".gdt")]
#[used]
static GDT: [GdtEntry; GDT_ENTRIES_COUNT] = [
    GdtEntry {
        limit0: 0,
        base0: 0,
        base1_flags: 0,
        limit1_flags_base2: 0,
    }, // Null segment
    make_entry(DESC_CODE32, 0, GDT_LIMIT),       // Kernel code
    make_entry(DESC_DATA32, 0, GDT_LIMIT),       // Kernel data
    make_entry(DESC_STACK32, 0, GDT_LIMIT),      // Kernel stack (expand-down data)
    make_entry(DESC_USER_CODE32, 0, GDT_LIMIT),  // User code
    make_entry(DESC_USER_DATA32, 0, GDT_LIMIT),  // User data
    make_entry(DESC_USER_STACK32, 0, GDT_LIMIT), // User stack (expand-down data)
];

pub fn load_gdt() {
    let gdt_ptr = GdtPointer {
        limit: (size_of::<[GdtEntry; GDT_ENTRIES_COUNT]>() - 1) as u16,
        base: &raw const GDT as *const _ as usize as u32,
    };

    unsafe {
        asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(nostack, preserves_flags)
        );

        asm!(
            // Set up data segment registers to kernel data selector.
            "mov ax, 0x10",
            "mov ds, ax",     // Move the value of ax into the data segment register
            "mov es, ax",     // Move the value of ax into the extra segment register
            "mov fs, ax",     // Move the value of ax into the fs segment register
            "mov gs, ax",     // Move the value of ax into the gs segment register
            "mov ss, ax",     // Move the value of ax into the stack segment register

            // Switch to kernel code selector with a far return.
            "push 0x08",
            "lea eax, [2f]",
            "push eax",
            "retf",
            "2:",
            out("eax") _,
        );
    }
}
