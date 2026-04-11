use bitflags::bitflags;
use spin::{Mutex, Once};

use crate::{
    memory::{
        alloc::Frame,
        paging::{Mapper, Page, PageFault},
    },
    panic::KernelFault,
};

pub const TABLE_ENTRIES: usize = 0x200;
static MAPPER: Once<Mutex<X86Mapper>> = Once::new();

bitflags! {
    pub struct PageFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABlE = 1 << 4;
        const NO_EXECUTE = 1 << 63;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn is_present(&self) -> bool {
        self.addr() & 1 != 0
    }

    pub fn addr(&self) -> usize {
        (self.0 as usize) & 0x000f_ffff_ffff_f000
    }

    pub fn set(&mut self, addr: usize, flags: PageFlags) {
        self.0 = (addr as u64) | flags.bits();
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

#[repr(C, align(0x1000))]
pub struct PageTable {
    entries: [PageTableEntry; TABLE_ENTRIES],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::empty(); TABLE_ENTRIES],
        }
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }
}

impl Page {
    pub fn pml4_index(self) -> usize {
        (self.addr() >> 39) & 0x1ff
    }

    pub fn pdpt_index(self) -> usize {
        (self.addr() >> 30) & 0x1ff
    }

    pub fn pd_index(self) -> usize {
        (self.addr() >> 21) & 0x1ff
    }

    pub fn pt_index(self) -> usize {
        (self.addr() >> 12) & 0x1ff
    }
}

pub struct X86Mapper {
    pml4: &'static mut PageTable,
}

impl X86Mapper {
    pub fn new(pml4: &'static mut PageTable) -> Self {
        Self { pml4 }
    }

    fn next_table(entry: &mut PageTableEntry) -> &'static mut PageTable {
        let table_ptr = entry.addr() as *mut PageTable;

        unsafe { &mut *table_ptr }
    }

    fn ensure_table(entry: &mut PageTableEntry) -> Result<&'static mut PageTable, KernelFault> {
        if !entry.is_present() {
            let frame = crate::memory::phys::frame_allocator()?
                .lock()
                .alloc()
                .map_err(|err| PageFault::OutOfFrames(err))?;

            let table = frame.addr() as *mut PageTable;

            unsafe {
                (*table).zero();
            }

            entry.set(frame.addr(), PageFlags::PRESENT | PageFlags::WRITABLE);
        }

        Ok(Self::next_table(entry))
    }
}

impl Mapper for X86Mapper {
    type PageFlags = PageFlags;

    fn map(&mut self, page: Page, frame: Frame, flags: PageFlags) -> Result<(), KernelFault> {
        let pml4e = &mut self.pml4.entries[page.pml4_index()];
        let pdpt = Self::ensure_table(pml4e);

        let pdpte = &mut pdpt?.entries[page.pdpt_index()];
        let pd = Self::ensure_table(pdpte);

        let pde = &mut pd?.entries[page.pd_index()];
        let pt = Self::ensure_table(pde);

        let pte = &mut pt?.entries[page.pt_index()];

        pte.set(frame.addr(), flags | PageFlags::PRESENT);

        Ok(())
    }

    fn unmap(&mut self, page: Page) -> Result<(), PageFault> {
        let idx = page.pt_index();
        if idx >= self.pml4.entries.len() {
            return Err(PageFault::InvalidAddress(idx));
        }
        self.pml4.entries[idx].clear();

        Ok(())
    }
}

pub fn init(pml4: &'static mut PageTable) {
    MAPPER.call_once(|| Mutex::new(X86Mapper::new(pml4)));
}

pub fn mapper() -> &'static Mutex<X86Mapper> {
    MAPPER.get().expect("Mapper not initialized")
}

pub fn allocate_pml4() -> Result<&'static mut PageTable, KernelFault> {
    let frame = crate::memory::phys::frame_allocator()?
        .lock()
        .alloc()
        .expect("Failed to allocate PML4");

    let pml4 = unsafe { &mut *(frame.addr() as *mut PageTable) };

    pml4.zero();

    Ok(pml4)
}
