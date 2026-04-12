use crate::cpu::interrupts::{
    exceptions::{ExceptionType, GenericExceptionInfo, handle_generic_exception},
    hardware::handle_hardware_interrupt,
};

pub mod exceptions;
pub mod hardware;

pub enum InterruptKind {
    Exception(ExceptionType),
    Hardware(u8), // The IRQ number (0-15)
}

pub struct GenericInterrupt {
    pub rip: u64,
    pub kind: InterruptKind,
}

pub fn handle_interrupt(info: GenericInterrupt) {
    match info.kind {
        InterruptKind::Exception(ex) => {
            // Re-use your existing logic!
            let ex_info = GenericExceptionInfo {
                rip: info.rip,
                exception: ex,
            };
            handle_generic_exception(ex_info);
        }
        InterruptKind::Hardware(irq) => {
            handle_hardware_interrupt(irq);
        }
    }
}
