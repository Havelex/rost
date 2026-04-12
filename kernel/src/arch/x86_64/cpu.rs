use crate::cpu::Cpu;

pub struct X86Cpu;

impl Cpu for X86Cpu {
    fn halt() -> ! {
        loop {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }

    fn nop() {
        unsafe { core::arch::asm!("nop") }
    }

    fn enable_interrupts() {
        unsafe { core::arch::asm!("sti") }
    }

    fn disable_interrupts() {
        unsafe { core::arch::asm!("cli") }
    }
}

pub trait X86CpuExt {
    fn read_cr2() -> usize;
}

impl X86CpuExt for X86Cpu {
    fn read_cr2() -> usize {
        let val: usize;
        unsafe {
            core::arch::asm!("mov {}, cr2", out(reg) val);
        }
        val
    }
}
