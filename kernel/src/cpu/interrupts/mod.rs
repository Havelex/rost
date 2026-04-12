#[derive(Debug)]
pub enum InterruptType {
    Timer,
    Keyboard,
    PageFault(u64), // The faulting address
    GeneralProtectionFault,
    DivideByZero,
}
