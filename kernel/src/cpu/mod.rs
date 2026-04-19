pub mod interrupts;

pub trait Cpu {
    fn halt() -> !;
    fn nop();
    fn enable_interrupts();
    fn disable_interrupts();
    /// Halt the CPU until the next interrupt fires, then return.
    ///
    /// On x86_64 this issues a single `hlt` instruction.  On other
    /// architectures an equivalent "wait for interrupt" instruction is used.
    fn wait_for_interrupt();
}
