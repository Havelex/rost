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
        println!("  [.] Initializing IDT...");
        idt::init();
        println!("  [*] IDT initialized.");
    }

    fn init_memory() {
        println!("  [.] Initializing paging...");

        let pml4 = memory::paging::allocate_pml4();
        memory::paging::init(pml4.expect("  [!] Fault at paging initialization"));

        println!("  [*] Paging initialized.");
    }

    fn init_interrupts() {}

    fn mapper() -> &'static Mutex<Self::Mapper> {
        mapper()
    }
}
