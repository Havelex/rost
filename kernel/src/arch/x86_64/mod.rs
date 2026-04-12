use spin::Mutex;

use crate::arch::{
    Architecture,
    x86_64::memory::paging::{X86Mapper, mapper},
};

mod gdt;
mod idt;
pub mod memory;
mod tss;

pub struct X86_64;

impl Architecture for X86_64 {
    type Mapper = X86Mapper;

    fn init_early() {
        println!("  [.] Initializing TSS...");
        tss::init();
        println!("  [*] TSS initialized.");
        println!("  [.] Initializing GDT...");
        gdt::init();
        println!("  [*] GDT initialized.");
    }

    fn halt() {
        // unsafe { core::arch::asm!("htl") }
    }

    fn init_interrupts() {
        println!("  [.] Initializing IDT...");
        idt::init();
        println!("  [*] IDT initialized.");
    }

    fn disable_interupts() {}

    fn enable_interrupts() {}

    fn init_memory() {
        println!("  [.] Initializing paging...");

        let pml4 = memory::paging::allocate_pml4();
        memory::paging::init(pml4.expect("  [!] Fault at paging initialization"));

        println!("  [*] Paging initialized.");
    }

    fn mapper() -> &'static Mutex<Self::Mapper> {
        mapper()
    }
}
