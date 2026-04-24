use crate::cpu::Cpu;

/// x86_64 interrupt controller and ISR glue implementation.
pub mod interrupts;

/// Zero-sized struct that implements [`Cpu`] for the x86_64 architecture.
///
/// All methods are associated functions (no `self` receiver) and delegate
/// directly to inline assembly instructions.
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

/// Extended CPU operations specific to the x86_64 architecture.
///
/// This trait supplements [`Cpu`] with x86_64-only functionality that is not
/// part of the portable [`Cpu`] interface.
pub trait X86CpuExt {
    /// Read the value of the CR2 register (the faulting virtual address after a page fault).
    ///
    /// # Returns
    /// The current value of CR2 as a `usize`.
    #[allow(dead_code)]
    fn read_cr2() -> usize;

    /// Enable SSE and SSE2 instructions by setting the required control-register bits.
    ///
    /// Must be called in [`init_early`](crate::arch::Architecture::init_early) before any
    /// code that may emit SSE instructions runs (the Rust compiler may generate SSE
    /// instructions when zeroing or copying large structures).
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
