use core::arch::global_asm;

use crate::error::Result;
global_asm!(include_str!(r#"interrupts.S"#), options(att_syntax));

#[derive(Clone, Copy)]
#[repr(C)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    _reserved: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            _reserved: 0,
        }
    }

    pub fn set_handler(&mut self, handler: *const ()) {
        let addr = handler as u64;
        self.offset_low = addr as u16;
        self.offset_mid = (addr >> 16) as u16;
        self.offset_high = (addr >> 32) as u32;
        self.selector = 0x08;
        self.flags = 0x8E;
        self.ist = 0;
        self._reserved = 0;
    }
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: u64,
}

unsafe extern "C" {
    static isr_stub_table: [extern "C" fn(); 48];
}

#[repr(C, align(16))]
struct IdtTable([IdtEntry; 256]);

static mut IDT: IdtTable = IdtTable([IdtEntry::missing(); 256]);

pub fn init() -> Result<()> {
    // Get a raw pointer to the inner array
    let idt_ptr = unsafe { &raw mut IDT.0 as *mut IdtEntry };

    for i in 0..48 {
        unsafe {
            let entry = &mut *idt_ptr.add(i);
            let handler_ptr = isr_stub_table[i] as *const ();
            entry.set_handler(handler_ptr);
        }
    }

    unsafe {
        // Set the IST for Double Fault
        IDT.0[8].ist = 1;
    }

    let idtr = Idtr {
        limit: (core::mem::size_of::<IdtTable>() - 1) as u16,
        base: &raw const IDT as u64,
    };

    unsafe {
        core::arch::asm!("lidt [{}]", in(reg) &idtr);
    }

    Ok(())
}
