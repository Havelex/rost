//! Generic CPU abstraction.
//!
//! Defines the [`Cpu`] trait that every architecture-specific CPU implementation
//! must satisfy so that the kernel can issue portable CPU control operations.

/// Architecture-independent CPU control interface.
///
/// Each target provides a zero-sized struct that implements this trait (e.g.
/// [`X86Cpu`](crate::arch::x86_64::cpu::X86Cpu) on x86_64).
pub mod interrupts;

/// Architecture-independent CPU control interface.
///
/// Each target provides a zero-sized struct that implements this trait.
pub trait Cpu {
    /// Halt the CPU in an infinite loop, waiting for the next interrupt.
    ///
    /// This function never returns.
    fn halt() -> !;

    /// Execute a single no-operation instruction.
    #[allow(dead_code)]
    fn nop();

    /// Unmask hardware interrupts on the current CPU.
    fn enable_interrupts();

    /// Mask hardware interrupts on the current CPU.
    fn disable_interrupts();
}
