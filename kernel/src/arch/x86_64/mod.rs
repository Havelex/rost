use spin::Mutex;

use crate::arch::{
    Architecture, Cpu,
    x86_64::{
        cpu::{X86Cpu, interrupts},
        interrupts::pic,
        memory::paging::{X86Mapper, mapper},
    },
};

mod cpu;
mod drivers;
mod gdt;
mod memory;
mod tss;

pub struct X86_64;

impl Architecture for X86_64 {
    type Mapper = X86Mapper;
    type Cpu = X86Cpu;

    fn init_early() {
        println!("  [.] Initializing TSS...");
        tss::init();
        println!("  [*] TSS initialized.");
        println!("  [.] Initializing GDT...");
        gdt::init();
        println!("  [*] GDT initialized.");
    }

    fn init_interrupts() {
        interrupts::init();
        unsafe { pic::remap_pic(32, 40) };
        drivers::pit::init(100);
        Self::Cpu::enable_interrupts();
    }

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
