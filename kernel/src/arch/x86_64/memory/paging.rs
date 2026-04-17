use bitflags::bitflags;
use spin::{Mutex, Once};
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    memory::{
        alloc::Frame,
        paging::{Mapper, Page, PageFault},
        regions::MemMap,
    },
    panic::KernelFault,
};

pub const TABLE_ENTRIES: usize = 0x200;
static MAPPER: Once<Mutex<X86Mapper>> = Once::new();

/// Physical-to-virtual offset provided by the Limine HHDM.
/// All physical frame addresses are accessible at `HHDM_OFFSET + phys`.
static HHDM_OFFSET: AtomicUsize = AtomicUsize::new(0);

/// Total physical memory size from the Limine memory map.
/// Used by `map_mmio_region` to detect whether an address has an existing
/// HHDM huge-page mapping.
static TOTAL_PHYS: AtomicUsize = AtomicUsize::new(0);

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct PageFlags: u64 {
        const PRESENT       = 1 << 0;
        const WRITABLE      = 1 << 1;
        const USER          = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABlE = 1 << 4;
        /// Page Size: when set on a PDPT entry → 1 GiB page;
        /// when set on a PD entry → 2 MiB page.
        const HUGE          = 1 << 7;
        const NO_EXECUTE    = 1 << 63;
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
        self.0 & 1 != 0
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
    pub entries: [PageTableEntry; TABLE_ENTRIES],
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

    /// Convert a physical address stored in a page-table entry to a virtual
    /// reference via the kernel HHDM.
    fn phys_to_virt(phys: usize) -> *mut PageTable {
        let offset = HHDM_OFFSET.load(Ordering::Acquire);
        (offset + phys) as *mut PageTable
    }

    fn next_table(entry: &mut PageTableEntry) -> &'static mut PageTable {
        unsafe { &mut *Self::phys_to_virt(entry.addr()) }
    }

    fn ensure_table(entry: &mut PageTableEntry) -> Result<&'static mut PageTable, KernelFault> {
        if !entry.is_present() {
            let frame = crate::memory::phys::frame_allocator()?
                .lock()
                .alloc()
                .map_err(PageFault::OutOfFrames)?;

            let phys = frame.addr();
            unsafe {
                (*Self::phys_to_virt(phys)).zero();
            }

            entry.set(phys, PageFlags::PRESENT | PageFlags::WRITABLE);
        }

        Ok(Self::next_table(entry))
    }
}

impl Mapper for X86Mapper {
    type PageFlags = PageFlags;

    fn map(&mut self, page: Page, frame: Frame, flags: PageFlags) -> Result<(), KernelFault> {
        let pml4e = &mut self.pml4.entries[page.pml4_index()];
        let pdpt = Self::ensure_table(pml4e)?;

        let pdpte = &mut pdpt.entries[page.pdpt_index()];
        let pd = Self::ensure_table(pdpte)?;

        let pde = &mut pd.entries[page.pd_index()];
        let pt = Self::ensure_table(pde)?;

        let pte = &mut pt.entries[page.pt_index()];
        pte.set(frame.addr(), flags | PageFlags::PRESENT);

        // Invalidate the TLB entry for this virtual page.
        unsafe {
            core::arch::asm!(
                "invlpg [{0}]",
                in(reg) page.addr(),
                options(nostack, preserves_flags),
            );
        }

        Ok(())
    }

    fn unmap(&mut self, page: Page) -> Result<(), PageFault> {
        let pml4e = &mut self.pml4.entries[page.pml4_index()];
        if !pml4e.is_present() {
            return Err(PageFault::Unmapped);
        }
        let pdpt = Self::next_table(pml4e);
        let pdpte = &mut pdpt.entries[page.pdpt_index()];
        if !pdpte.is_present() {
            return Err(PageFault::Unmapped);
        }
        let pd = Self::next_table(pdpte);
        let pde = &mut pd.entries[page.pd_index()];
        if !pde.is_present() {
            return Err(PageFault::Unmapped);
        }
        let pt = Self::next_table(pde);
        let pte = &mut pt.entries[page.pt_index()];
        pte.clear();

        unsafe {
            core::arch::asm!(
                "invlpg [{0}]",
                in(reg) page.addr(),
                options(nostack, preserves_flags),
            );
        }

        Ok(())
    }
}

/// Initialise the global mapper with the given (HHDM-virtual) PML4 reference.
/// Must be called after `init_paging`.
pub fn init(pml4: &'static mut PageTable) {
    MAPPER.call_once(|| Mutex::new(X86Mapper::new(pml4)));
}

pub fn mapper() -> &'static Mutex<X86Mapper> {
    MAPPER.get().expect("Mapper not initialized")
}

// ──────────────────────────────────────────────────────────────────────────────
// Paging initialisation
// ──────────────────────────────────────────────────────────────────────────────

/// Allocate a zeroed 4 KiB physical frame and return a static mutable
/// reference to it through the HHDM.
fn alloc_page_table() -> Result<&'static mut PageTable, KernelFault> {
    let frame = crate::memory::phys::frame_allocator()?
        .lock()
        .alloc()
        .map_err(PageFault::OutOfFrames)?;
    let phys = frame.addr();
    let offset = HHDM_OFFSET.load(Ordering::Acquire);
    let virt = (offset + phys) as *mut PageTable;
    unsafe {
        (*virt).zero();
        Ok(&mut *virt)
    }
}

/// Return the physical address of a HHDM-virtual page-table pointer.
fn virt_to_phys(virt: *const PageTable) -> usize {
    let offset = HHDM_OFFSET.load(Ordering::Acquire);
    virt as usize - offset
}

/// Map a single 1 GiB huge page: `virt` → `phys` (PDPT-level PS=1).
fn map_1gib(pml4: &mut PageTable, virt: usize, phys: usize) -> Result<(), KernelFault> {
    let pml4e = &mut pml4.entries[(virt >> 39) & 0x1ff];
    if !pml4e.is_present() {
        let pdpt = alloc_page_table()?;
        pml4e.set(virt_to_phys(pdpt), PageFlags::PRESENT | PageFlags::WRITABLE);
    }
    let pdpt = unsafe { &mut *X86Mapper::phys_to_virt(pml4e.addr()) };
    let pdpte = &mut pdpt.entries[(virt >> 30) & 0x1ff];
    pdpte.set(phys, PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::HUGE);
    Ok(())
}

/// Map a single 4 KiB page: `virt` → `phys`.
fn map_4kib(
    pml4: &mut PageTable,
    virt: usize,
    phys: usize,
    flags: PageFlags,
) -> Result<(), KernelFault> {
    let pml4e = &mut pml4.entries[(virt >> 39) & 0x1ff];
    if !pml4e.is_present() {
        let pdpt = alloc_page_table()?;
        pml4e.set(virt_to_phys(pdpt), PageFlags::PRESENT | PageFlags::WRITABLE);
    }
    let pdpt = unsafe { &mut *X86Mapper::phys_to_virt(pml4e.addr()) };

    let pdpte = &mut pdpt.entries[(virt >> 30) & 0x1ff];
    if !pdpte.is_present() {
        let pd = alloc_page_table()?;
        pdpte.set(virt_to_phys(pd), PageFlags::PRESENT | PageFlags::WRITABLE);
    }
    let pd = unsafe { &mut *X86Mapper::phys_to_virt(pdpte.addr()) };

    let pde = &mut pd.entries[(virt >> 21) & 0x1ff];
    if !pde.is_present() {
        let pt = alloc_page_table()?;
        pde.set(virt_to_phys(pt), PageFlags::PRESENT | PageFlags::WRITABLE);
    }
    let pt = unsafe { &mut *X86Mapper::phys_to_virt(pde.addr()) };

    let pte = &mut pt.entries[(virt >> 12) & 0x1ff];
    pte.set(phys, flags | PageFlags::PRESENT);
    Ok(())
}

/// Build a fresh set of page tables, load CR3, and initialise the global mapper.
///
/// Steps:
/// 1. Record the HHDM offset so frame accesses go through the correct virtual
///    address.
/// 2. Map all physical memory via 1 GiB HHDM huge pages.
/// 3. Map the kernel image (4 KiB pages) from its physical base to its
///    higher-half virtual base.
/// 4. Load CR3 with the new PML4 physical address.
/// 5. Verify the mapping and register the global mapper.
pub fn init_paging(
    hhdm_offset: usize,
    kernel_phys_base: usize,
    kernel_virt_base: usize,
    mem_map: &MemMap,
) -> Result<(), KernelFault> {
    // 1. Record the HHDM offset so alloc_page_table / phys_to_virt work.
    HHDM_OFFSET.store(hhdm_offset, Ordering::Release);

    // 2. Allocate the top-level PML4 via the HHDM.
    let pml4_phys = {
        let frame = crate::memory::phys::frame_allocator()?
            .lock()
            .alloc()
            .map_err(PageFault::OutOfFrames)?;
        frame.addr()
    };
    let pml4_virt = (hhdm_offset + pml4_phys) as *mut PageTable;
    unsafe { (*pml4_virt).zero() };
    let pml4 = unsafe { &mut *pml4_virt };

    // 3. Map all physical memory with 1 GiB huge pages so that every frame is
    //    reachable after we switch CR3.
    let total_phys = mem_map.total_mem_size;
    TOTAL_PHYS.store(total_phys, Ordering::Release);
    const GIB: usize = 1 << 30;
    let num_gib = (total_phys + GIB - 1) / GIB;
    for i in 0..num_gib {
        let phys = i * GIB;
        let virt = hhdm_offset + phys;
        map_1gib(pml4, virt, phys)?;
    }

    // 4. Determine kernel size from the memory map (KernelAndModules entry).
    let kernel_end_phys = mem_map
        .regions
        .iter()
        .take(mem_map.count)
        .filter(|r| {
            matches!(r.kind, crate::memory::regions::MemoryRegionKind::KernelAndModules)
        })
        .map(|r| r.base + r.length)
        .max()
        .unwrap_or(kernel_phys_base);

    let kernel_size = kernel_end_phys
        .saturating_sub(kernel_phys_base)
        .max(8 * 1024 * 1024); // at least 8 MiB

    // 5. Map the kernel image (PRESENT | WRITABLE).
    const PAGE: usize = 0x1000;
    let num_pages = (kernel_size + PAGE - 1) / PAGE;
    for i in 0..num_pages {
        let phys = kernel_phys_base + i * PAGE;
        let virt = kernel_virt_base + i * PAGE;
        map_4kib(pml4, virt, phys, PageFlags::WRITABLE)?;
    }

    // 6. Load the new CR3 (flush TLB completely).
    log_info!("  [paging] HHDM offset: {:#018x}", hhdm_offset);
    log_info!("  [paging] PML4 phys:   {:#018x}", pml4_phys);

    unsafe {
        core::arch::asm!(
            "mov cr3, {0}",
            in(reg) pml4_phys,
            options(nostack, preserves_flags),
        );
    }

    // 7. Verify: read a byte through the HHDM mapping we just built.
    let test_virt = hhdm_offset as *const u8;
    let _test_byte = unsafe { test_virt.read_volatile() };
    log_ok!(
        "  [paging] HHDM read-back OK (phys 0x0 → virt {:#018x})",
        hhdm_offset
    );

    // 8. Register the global mapper.
    init(unsafe { &mut *pml4_virt });

    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// MMIO mapping helper
// ──────────────────────────────────────────────────────────────────────────────

/// Map `size` bytes of MMIO starting at physical address `phys_start` into the
/// kernel's virtual address space via the HHDM.
///
/// Returns the virtual address corresponding to `phys_start`.
///
/// For physical addresses within the HHDM-mapped RAM range (already covered by
/// 1 GiB huge pages), the existing mapping is reused and the virtual address is
/// returned.  Cache-disable for in-HHDM MMIO requires splitting the huge pages
/// and is a future enhancement.
///
/// For physical addresses outside the RAM range (true MMIO space), fresh 4 KiB
/// page-table entries are added with PRESENT | WRITABLE | NO_EXECUTE |
/// CACHE_DISABLE flags.
pub fn map_mmio_region(phys_start: usize, size: usize) -> Result<usize, KernelFault> {
    let hhdm = HHDM_OFFSET.load(Ordering::Acquire);
    let total_phys = TOTAL_PHYS.load(Ordering::Acquire);
    const PAGE: usize = 0x1000;

    // Check whether the entire MMIO region falls within the HHDM-mapped RAM.
    // If any page extends beyond total_phys, we must add explicit mappings for
    // those pages.
    let phys_base = phys_start & !(PAGE - 1);
    let offset_in_page = phys_start - phys_base;
    let num_pages = (offset_in_page + size + PAGE - 1) / PAGE;

    let flags = PageFlags::WRITABLE | PageFlags::NO_EXECUTE | PageFlags::CACHE_DISABlE;

    {
        let mut m = mapper().lock();
        for i in 0..num_pages {
            let phys = phys_base + i * PAGE;
            if phys >= total_phys {
                // Outside HHDM range: add a new 4 KiB mapping with
                // cache-disable flags so MMIO registers are not cached.
                m.map(Page::new(hhdm + phys), Frame::new(phys), flags)?;
            }
            // For phys < total_phys the 1 GiB HHDM huge-page already covers
            // this address.  Splitting the huge page for cache-disable is
            // deferred to when the APIC/IOAPIC driver is initialised.
        }
    }

    Ok(hhdm + phys_start)
}
