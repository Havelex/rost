use core::mem::MaybeUninit;

use crate::error::Result;

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
            iopb_offset: 0,
        }
    }
}

static mut TSS: MaybeUninit<Tss> = MaybeUninit::uninit();
static DF_STACK: [u8; 4096] = [0; 4096];

pub fn init() -> Result<()> {
    let mut tss = Tss::new();

    tss.ist[0] = unsafe { (&raw const DF_STACK as *const u8).add(4096) as u64 };

    unsafe {
        core::ptr::write(&raw mut TSS as *mut MaybeUninit<Tss>, MaybeUninit::new(tss));
    }

    Ok(())
}

pub fn get() -> &'static Tss {
    unsafe { (&raw const TSS).cast::<Tss>().as_ref().unwrap() }
}
