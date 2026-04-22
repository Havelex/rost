//! Paging management.
//!
//! This module provides the `Mapper` trait for mapping and unmapping pages to frames, as well as
//! the `Page` struct for representing pages and the `PageFault` enum for handling paging errors.

use crate::{
    memory::alloc::{Frame, MemoryFault},
    panic::KernelFault,
};

const PAGE_SIZE: usize = 0x1000;

/// Enum representing various page faults that can occur during paging operations.
#[allow(dead_code)]
#[derive(Debug)]
pub enum PageFault {
    AlreadyMapped,
    Unmapped,
    InvalidAddress(usize),
    OutOfFrames(MemoryFault),
}

impl From<PageFault> for KernelFault {
    fn from(err: PageFault) -> Self {
        KernelFault::Paging(err)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Page(pub usize);

impl Page {
    pub const fn new(addr: usize) -> Self {
        Self(addr & !(PAGE_SIZE - 1))
    }

    pub const fn addr(self) -> usize {
        self.0
    }

    #[allow(dead_code)]
    pub const fn index(self) -> usize {
        self.0 / PAGE_SIZE
    }
}

pub trait Mapper {
    type PageFlags;

    fn map(&mut self, page: Page, frame: Frame, flags: Self::PageFlags) -> Result<(), KernelFault>;

    #[allow(dead_code)]
    fn unmap(&mut self, page: Page) -> Result<(), PageFault>;
}
