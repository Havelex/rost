use crate::error::Result;

use super::tss;

#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}

static mut GDT: [u64; 5] = [0; 5];

pub fn init() -> Result<()> {
    unsafe {
        GDT[0] = 0;
        GDT[1] = 0x00AF9A000000FFFF; // kernel code
        GDT[2] = 0x00AF92000000FFFF; // kernel data

        let tss_ptr = tss::get() as *const _ as u64;
        let tss_limit = core::mem::size_of::<tss::Tss>() as u64 - 1;

        GDT[3] = (tss_limit & 0xFFFF)
            | ((tss_ptr & 0xFFFFFF) << 16)
            | (0x89 << 40)
            | ((tss_limit & 0xF0000) << 32)
            | ((tss_ptr & 0xFF000000) << 32);

        GDT[4] = tss_ptr >> 32;

        let gdtr = GdtDescriptor {
            limit: (core::mem::size_of::<[u64; 5]>() - 1) as u16,
            base: (&raw const GDT) as u64,
        };

        core::arch::asm!("lgdt [{}]", in(reg) &gdtr);

        core::arch::asm!(
            "push 0x08",           // Push the Code Segment selector (Index 1)
            "lea rax, [rip + 2f]", // Load address of label '2' into RAX
            "push rax",            // Push it as the return RIP
            "retfq",               // Far return: pops RAX into RIP and 0x08 into CS
            "2:",                  // The "Return" point
            "mov ax, 0x10",        // Data segment (Index 2)
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
        );

        core::arch::asm!("ltr ax", in("ax") 0x18u16);
    }

    Ok(())
}
