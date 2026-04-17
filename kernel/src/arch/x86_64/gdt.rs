use super::tss;
use crate::error::Result;
use core::ptr::addr_of_mut;

#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    flags_limit_high: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn new(access: u8, flags: u8) -> Self {
        GdtEntry {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access,
            flags_limit_high: (flags & 0xF0) | 0x0F,
            base_high: 0,
        }
    }
}

#[repr(C, align(16))]
struct GdtTable {
    entries: [GdtEntry; 3],
    tss_low: u64,
    tss_high: u64,
}

static mut GDT: GdtTable = GdtTable {
    entries: [GdtEntry {
        limit_low: 0,
        base_low: 0,
        base_mid: 0,
        access: 0,
        flags_limit_high: 0,
        base_high: 0,
    }; 3],
    tss_low: 0,
    tss_high: 0,
};

static mut GDTR: GdtDescriptor = GdtDescriptor { limit: 0, base: 0 };

pub fn init() -> Result<()> {
    unsafe {
        let gdt = addr_of_mut!(GDT);

        // 1. Kernel Code: Access 0x9A, Flags 0xA0
        (*gdt).entries[1] = GdtEntry::new(0x9A, 0xA0);

        // 2. Kernel Data: Access 0x92, Flags 0xC0
        (*gdt).entries[2] = GdtEntry::new(0x92, 0xC0);

        // --- TSS Setup ---
        let tss_ptr = tss::get() as *const _ as u64;
        let tss_limit = (core::mem::size_of::<tss::Tss>() - 1) as u64;

        (*gdt).tss_low = (tss_limit & 0xFFFF)
            | ((tss_ptr & 0xFFFFFF) << 16)
            | (0x89 << 40)
            | ((tss_limit & 0xF0000) << 32)
            | ((tss_ptr & 0xFF000000) << 32);

        (*gdt).tss_high = tss_ptr >> 32;

        // --- Load Tables ---
        let gdtr = addr_of_mut!(GDTR);
        (*gdtr).limit = (core::mem::size_of::<GdtTable>() - 1) as u16;
        (*gdtr).base = gdt as u64;

        core::arch::asm!(
            "lgdt [{0}]",
            "push 0x08",          // CS Selector
            "lea rax, [rip + 2f]", // Jump to label 2 forward
            "push rax",
            "retfq",              // Far return to reload CS
            "2:",                 // Using '2' to avoid binary literal ambiguity
            "mov ax, 0x10",       // DS Selector
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
            "ltr {1:x}",          // TSS Selector (0x18)
            in(reg) gdtr,
            in(reg) 0x18u16,
            out("rax") _,
        );
    }
    Ok(())
}
