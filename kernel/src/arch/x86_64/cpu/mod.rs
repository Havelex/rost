use crate::cpu::Cpu;

pub(super) mod interrupts;

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
        unsafe {
            core::arch::asm!("sti", options(nomem, nostack));
        }
    }

    fn disable_interrupts() {
        unsafe {
            core::arch::asm!("cli", options(nomem, nostack));
        }
    }

    fn send_eoi(irq: u8) {
        if irq >= 8 {
            outb(0xA0, 0x20);
        }
        outb(0x20, 0x20);
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

#[inline(always)]
pub fn outb(port: u16, val: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub fn inb(port: u16) -> u8 {
    let res: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            out("al") res,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    res
}
