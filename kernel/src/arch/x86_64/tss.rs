//! x86_64 Task State Segment (TSS).
//!
//! Provides the [`Tss`] structure laid out according to the Intel/AMD 64-bit
//! long-mode specification, a dedicated double-fault stack, and helpers to
//! initialize and retrieve the single global TSS instance.

use crate::error::Result;
use core::mem::MaybeUninit;

/// The x86_64 Task State Segment as defined in the Intel / AMD64 architecture manuals.
///
/// Only the fields relevant to the kernel (ring-0 stack pointer, IST entries,
/// and the I/O permission bitmap offset) are exposed publicly.  All reserved
/// fields are kept private and must remain zero.
#[repr(C, packed)]
pub struct Tss {
    _reserved1: u32,
    /// Ring-0, 1, and 2 stack pointers loaded by the CPU on privilege-level changes.
    ///
    /// `rsp[0]` is the ring-0 stack pointer used on every interrupt taken from
    /// user mode (ring 3).
    pub rsp: [u64; 3],
    _reserved2: u64,
    /// Interrupt Stack Table entries.
    ///
    /// An IDT gate can reference one of these entries (IST 1–7) to switch to a
    /// known-good stack unconditionally.  `ist[0]` (IST 1) is used for the
    /// double-fault handler.
    pub ist: [u64; 7],
    _reserved3: u64,
    _reserved4: u16,
    /// Offset of the I/O Permission Bitmap from the base of the TSS.
    ///
    /// Set to `size_of::<Tss>()` to indicate that no I/O bitmap is present,
    /// which blocks all ring-3 direct port I/O.
    pub iopb_offset: u16,
}

impl Tss {
    /// Create a zeroed TSS with the IOPB offset set past the end of the struct.
    ///
    /// The IOPB offset is set to `size_of::<Tss>()` so that no I/O bitmap is
    /// present; all port I/O from user mode is disallowed.
    pub const fn new() -> Self {
        Self {
            _reserved1: 0,
            rsp: [0; 3],
            _reserved2: 0,
            ist: [0; 7],
            _reserved3: 0,
            _reserved4: 0,
            // Must point to or beyond the end of the TSS to disable I/O bitmap
            iopb_offset: core::mem::size_of::<Tss>() as u16,
        }
    }
}

static mut TSS: MaybeUninit<Tss> = MaybeUninit::uninit();

// Align to 16 bytes for ABI compliance
#[repr(C, align(16))]
struct Stack([u8; 4096]);
static mut DF_STACK: Stack = Stack([0; 4096]);

use core::ptr::{addr_of, addr_of_mut};

/// Initialize the global TSS and set up the double-fault IST stack.
///
/// Writes a fresh [`Tss`] into the global `TSS` static, points `ist[0]` (IST 1)
/// to the top of a dedicated 4 KiB stack, and stores the same address in `rsp[0]`
/// for use on ring-3 → ring-0 transitions.
///
/// # Returns
/// - `Ok(())` on success.
pub fn init() -> Result<()> {
    let mut tss = Tss::new();

    unsafe {
        // Point IST[0] to the top of the stack
        let stack_ptr = (&raw mut DF_STACK).cast::<u8>().add(4096);
        tss.ist[0] = stack_ptr as u64;
        tss.rsp[0] = stack_ptr as u64;

        // Use addr_of_mut! to get a raw pointer without creating a reference
        let tss_ptr = addr_of_mut!(TSS);
        core::ptr::write(tss_ptr.cast::<Tss>(), tss);
    }

    Ok(())
}

/// Return a reference to the initialized global TSS.
///
/// # Returns
/// A `'static` reference to the [`Tss`] written by [`init`].
///
/// # Panics
/// Behaviour is undefined (not a panic) if called before [`init`].  Always
/// call [`init`] exactly once during early boot before using this function.
pub fn get() -> &'static Tss {
    unsafe {
        // We cast the raw pointer to a reference at the very last second.
        // This is still unsafe, but it satisfies the compiler's new rules.
        &*addr_of!(TSS).cast::<Tss>()
    }
}
