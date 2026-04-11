use crate::{memory::alloc::Frame, panic::KernelFault};

pub const PAGE_SIZE: usize = 0x1000;

pub enum PageFault {
    AlreadyMapped,
    Unmapped,
    InvalidAddress(usize),
    OutOfFrames,
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

    pub const fn index(self) -> usize {
        self.0 / PAGE_SIZE
    }
}

pub trait Mapper {
    type PageFlags;

    fn map(&mut self, page: Page, frame: Frame, flags: Self::PageFlags) -> Result<(), PageFault>;

    fn unmap(&mut self, page: Page) -> Result<(), PageFault>;
}
