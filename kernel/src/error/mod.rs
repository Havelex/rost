//! Kernel error types.
//!
//! Defines the [`KernelError`] enum used as the `Err` variant in all fallible
//! kernel operations, and the [`Result`] type alias that wraps it.

/// A specialized [`Result`](core::result::Result) type for kernel operations.
///
/// The `Ok` variant carries the success value `T`; the `Err` variant always
/// carries a [`KernelError`].
pub type Result<T> = core::result::Result<T, KernelError>;

/// Errors that can occur during kernel initialization or operation.
#[derive(Debug, Clone, Copy)]
pub enum KernelError {
    /// The Global Descriptor Table could not be initialized.
    GdtInitFailed,
    /// The Interrupt Descriptor Table could not be initialized.
    IdtInitFailed,
    /// The Task State Segment could not be initialized.
    TssInitFailed,
    /// A memory allocation failed because no physical frames are available.
    OutOfMemory,
    /// A generic error with a human-readable description.
    Generic(&'static str),
}
