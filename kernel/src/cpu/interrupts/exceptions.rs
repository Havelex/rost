//! CPU exception types and handler.
//!
//! Defines the [`ExceptionType`] enum for all CPU exceptions recognised by the
//! kernel and the [`handle_generic_exception`] dispatcher that acts on them.

/// CPU exception type, with variant-specific payload where applicable.
pub enum ExceptionType {
    /// Vector 0 — division by zero or integer divide overflow (`#DE`).
    DivideByZero,
    /// Vector 8 — double fault; fires when a second exception occurs while
    /// handling a prior exception (`#DF`).
    DoubleFault,
    /// Vector 3 — software breakpoint (`#BP`); typically triggered by `int3`.
    Breakpoint,
    /// Vector 13 — general protection fault (`#GP`); carries the segment
    /// selector error code (0 if not selector-related).
    GeneralProtectionFault(u64),
    /// Vector 14 — page fault (`#PF`).
    ///
    /// `addr` is the faulting virtual address (CR2); `error_code` encodes the
    /// fault type (present/write/user/fetch — see the Intel SDM).
    PageFault { addr: u64, error_code: u64 },
    /// An exception vector not explicitly handled by the kernel.
    Unknown(u64),
}

/// Bundled information passed to [`handle_generic_exception`].
pub struct GenericExceptionInfo {
    /// Instruction pointer at the time the exception fired.
    pub rip: u64,
    /// The specific exception that occurred.
    pub exception: ExceptionType,
}

/// Dispatch a CPU exception to the appropriate handler.
///
/// Breakpoints are logged and handled non-fatally; all other exceptions
/// currently trigger a kernel panic.
///
/// # Parameters
/// - `info`: The exception descriptor produced by the architecture-specific handler.
pub fn handle_generic_exception(info: GenericExceptionInfo) {
    match info.exception {
        ExceptionType::Breakpoint => {
            log_info!("Stopping at breakpoint: RIP={:#x}", info.rip);
        }
        ExceptionType::DoubleFault => {
            panic!("Double Fault at {:#x}", info.rip)
        }
        ExceptionType::PageFault { addr, error_code } => {
            panic!(
                "PAGE FAULT at {:#x}\nAttempted to access: {:#x}\nError Flags: {:#b}",
                info.rip, addr, error_code
            );
        }
        ExceptionType::GeneralProtectionFault(code) => {
            panic!("GPF at {:#x} with error code {:#x}", info.rip, code);
        }
        ExceptionType::Unknown(vector) => {
            panic!(
                "Unkown CPU exception at {:#x} with vector {:#x}",
                info.rip, vector
            )
        }
        _ => panic!("Unhandled exception at {:#x}", info.rip),
    }
}
