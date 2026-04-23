//! Paging management.
//!
//! This module provides the `Mapper` trait for mapping and unmapping pages to frames, as well as
//! the `Page` struct for representing pages and the `PageFault` enum for handling paging errors.

use crate::{
    memory::alloc::{Frame, MemoryFault},
    panic::KernelFault,
};

const PAGE_SIZE: usize = 0x1000;

/// Errors produced during page-mapping operations.
#[allow(dead_code)]
#[derive(Debug)]
pub enum PageFault {
    /// A page was already mapped when a new mapping was requested.
    AlreadyMapped,
    /// A page was not mapped when an operation required it to be.
    Unmapped,
    /// The virtual address passed to a mapping operation is not page-aligned or is otherwise invalid.
    InvalidAddress(usize),
    /// Physical frame allocation failed while creating a page-table entry.
    OutOfFrames(MemoryFault),
}

impl From<PageFault> for KernelFault {
    fn from(err: PageFault) -> Self {
        KernelFault::Paging(err)
    }
}

/// A 4 KiB virtual memory page, identified by its page-aligned address.
#[derive(Clone, Copy, Debug)]
pub struct Page(pub usize);

impl Page {
    /// Create a [`Page`] from `addr`, aligning it down to the nearest page boundary.
    ///
    /// # Parameters
    /// - `addr`: Any virtual address within the target page.
    ///
    /// # Returns
    /// A [`Page`] whose base address is `addr` rounded down to a 4 KiB boundary.
    pub const fn new(addr: usize) -> Self {
        Self(addr & !(PAGE_SIZE - 1))
    }

    /// Return the page-aligned base address.
    ///
    /// # Returns
    /// The virtual address of the first byte of this page.
    pub const fn addr(self) -> usize {
        self.0
    }

    /// Return the page index (base address divided by [`PAGE_SIZE`]).
    ///
    /// # Returns
    /// The zero-based index of this page in the virtual address space.
    #[allow(dead_code)]
    pub const fn index(self) -> usize {
        self.0 / PAGE_SIZE
    }
}

/// Abstraction over a hardware page-table walker.
///
/// Implementors provide [`map`](Mapper::map) and [`unmap`](Mapper::unmap)
/// operations for associating virtual pages with physical frames.
pub trait Mapper {
    /// Architecture-specific page-mapping flags (e.g. present, writable, user).
    type PageFlags;

    /// Map `page` to `frame` with the given `flags`.
    ///
    /// # Parameters
    /// - `page`: The virtual page to map.
    /// - `frame`: The physical frame to back the page with.
    /// - `flags`: Architecture-specific flags controlling access permissions and cacheability.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(KernelFault)` if the mapping could not be created.
    fn map(&mut self, page: Page, frame: Frame, flags: Self::PageFlags) -> Result<(), KernelFault>;

    /// Unmap `page`, removing its entry from the page table.
    ///
    /// # Parameters
    /// - `page`: The virtual page to unmap.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(PageFault::Unmapped)` if the page was not mapped.
    #[allow(dead_code)]
    fn unmap(&mut self, page: Page) -> Result<(), PageFault>;
}
