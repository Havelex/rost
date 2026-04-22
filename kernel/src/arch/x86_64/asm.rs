//! Low-level x86_64 I/O port primitives.
//!
//! Provides `outb` and `inb` wrappers around the `OUT` and `IN` machine
//! instructions for communicating with memory-mapped I/O ports.

use core::arch::asm;

/// Write a single byte to an x86_64 I/O port.
///
/// # Parameters
/// - `port`: The 16-bit I/O port address to write to.
/// - `val`: The byte value to write.
///
/// # Safety
/// The caller must ensure that:
/// - `port` is a valid I/O port for the current hardware configuration.
/// - Writing `val` to `port` is safe in the current execution context (e.g.
///   the port is not shared with an interrupt handler that could fire
///   concurrently on a multi-processor system).
#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",  // Intel syntax: out DEST (port), SRC (value)
            in("dx") port,
            in("al") val,
            options(nostack, preserves_flags)
        );
    }
}

/// Read a single byte from an x86_64 I/O port.
///
/// # Parameters
/// - `port`: The 16-bit I/O port address to read from.
///
/// # Returns
/// The byte value read from the port.
///
/// # Safety
/// The caller must ensure that:
/// - `port` is a valid I/O port for the current hardware configuration.
/// - Reading from `port` is safe in the current execution context.
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let res: u8;
    unsafe {
        asm!(
            "in al, dx",   // Intel syntax: in DEST (value), SRC (port)
            in("dx") port,
            out("al") res,
            options(nostack, preserves_flags)
        );
    }
    res
}
