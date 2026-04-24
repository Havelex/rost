//! Generic interrupt handling.
//!
//! Defines the portable [`InterruptKind`] / [`GenericInterrupt`] types and the
//! [`handle_interrupt`] dispatcher that routes interrupts to the appropriate
//! exception or hardware handler.

use crate::cpu::interrupts::{
    exceptions::{ExceptionType, GenericExceptionInfo, handle_generic_exception},
    hardware::handle_hardware_interrupt,
};

/// CPU-exception definitions and generic exception dispatcher.
pub mod exceptions;
/// Hardware IRQ dispatcher and device-specific interrupt handling.
pub mod hardware;

/// Classifies an interrupt as either a CPU exception or a hardware IRQ.
pub enum InterruptKind {
    /// A synchronous CPU exception (e.g. page fault, divide-by-zero).
    Exception(ExceptionType),
    /// An asynchronous hardware interrupt; carries the IRQ line number (0–15).
    Hardware(u8),
}

/// Architecture-independent interrupt descriptor passed to [`handle_interrupt`].
pub struct GenericInterrupt {
    /// Instruction pointer at the time the interrupt fired.
    pub rip: u64,
    /// Whether this is a CPU exception or a hardware IRQ, and which one.
    pub kind: InterruptKind,
}

/// Route a [`GenericInterrupt`] to the correct handler.
///
/// CPU exceptions are forwarded to [`handle_generic_exception`]; hardware IRQs
/// are forwarded to [`handle_hardware_interrupt`].
///
/// # Parameters
/// - `info`: The interrupt descriptor produced by the architecture-specific stub.
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
