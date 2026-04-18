use crate::cpu::Cpu;

pub mod interrupts;

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
}

pub trait X86CpuExt {
    fn read_cr2() -> usize;
    fn enable_sse();
}

impl X86CpuExt for X86Cpu {
    fn read_cr2() -> usize {
        let val: usize;
        unsafe {
            core::arch::asm!("mov {}, cr2", out(reg) val);
        }
        val
    }

    /// Enable SSE and SSE2 instructions.
    ///
    /// x86_64 mandates SSE2, but the OS must explicitly signal that it manages
    /// the FXSAVE/FXRSTOR state before the CPU will execute SSE instructions:
    ///
    ///   - CR0.MP (bit 1): monitor coprocessor (needed so that WAIT/FWAIT
    ///     check CR0.TS; also required before setting OSFXSR).
    ///   - CR0.EM (bit 2): must be *clear* (no x87 emulation) – it is already
    ///     clear after Limine transitions to long mode, but we clear it
    ///     explicitly to be safe.
    ///   - CR4.OSFXSR (bit 9): OS provides FXSAVE/FXRSTOR support → unlocks
    ///     SSE instructions; without this they #UD.
    ///   - CR4.OSXMMEXCPT (bit 10): OS can handle SIMD floating-point
    ///     exceptions (#XM/vector 19) instead of routing them through #UD.
    fn enable_sse() {
        unsafe {
            let mut cr0: u64;
            core::arch::asm!("mov {}, cr0", out(reg) cr0);
            cr0 &= !(1u64 << 2); // clear CR0.EM
            cr0 |=  1u64 << 1;   // set   CR0.MP
            core::arch::asm!("mov cr0, {}", in(reg) cr0, options(nostack));

            let mut cr4: u64;
            core::arch::asm!("mov {}, cr4", out(reg) cr4);
            cr4 |= (1u64 << 9) | (1u64 << 10); // set OSFXSR and OSXMMEXCPT
            core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nostack));
        }
    }
}
