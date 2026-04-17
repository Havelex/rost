use crate::error::Result;
use core::mem::MaybeUninit;

#[repr(C, packed)]
pub struct Tss {
    _reserved1: u32,
    pub rsp: [u64; 3],
    _reserved2: u64,
    pub ist: [u64; 7],
    _reserved3: u64,
    _reserved4: u16,
    pub iopb_offset: u16,
}

impl Tss {
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

pub fn get() -> &'static Tss {
    unsafe {
        // We cast the raw pointer to a reference at the very last second.
        // This is still unsafe, but it satisfies the compiler's new rules.
        &*addr_of!(TSS).cast::<Tss>()
    }
}
