pub type Result<T> = core::result::Result<T, KernelError>;

#[derive(Debug, Clone, Copy)]
pub enum KernelError {
    GdtInitFailed,
    IdtInitFailed,
    TssInitFailed,
    OutOfMemory,
    Generic(&'static str),
}
