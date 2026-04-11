use super::tss;

#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}

static mut GDT: [u64; 5] = [0; 5];

pub fn init() {
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
            base: (&raw const GDT) as *const _ as u64,
        };

        core::arch::asm!("lgdt [{}]", in(reg) &gdtr);

        core::arch::asm!(
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
        );

        core::arch::asm!("ltr ax", in("ax") 0x18u16);
    }
}
