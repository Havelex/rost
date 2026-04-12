pub mod exceptions;
pub mod interrupts;

pub trait Cpu {
    fn halt() -> !;
    fn nop();
    fn enable_interrupts();
    fn disable_interrupts();
}
