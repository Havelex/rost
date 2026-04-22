pub mod interrupts;

pub trait Cpu {
    fn halt() -> !;
    #[allow(dead_code)]
    fn nop();
    fn enable_interrupts();
    fn disable_interrupts();
}
