use crate::panic::KernelFault;

pub const FRAME_SIZE: usize = 0x1000;

#[derive(Debug)]
pub enum MemoryFault {
    FrameIndexOutOfBounds { idx: usize, max: usize },
    DoubleAllocation { idx: usize },
    DoubleFree { idx: usize },
    OutOfMemory,
    NoAllocator,
}

impl From<MemoryFault> for KernelFault {
    fn from(err: MemoryFault) -> Self {
        KernelFault::Memory(err)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frame(usize);

impl Frame {
    pub const fn new(addr: usize) -> Self {
        Self(addr & !(FRAME_SIZE - 1))
    }

    pub const fn from_index(idx: usize) -> Self {
        Self(idx * FRAME_SIZE)
    }

    pub const fn addr(self) -> usize {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 / FRAME_SIZE
    }
}

pub struct FrameAllocator {
    bitmap: &'static mut [u64],
    total_frames: usize,
}

impl FrameAllocator {
    pub fn new(bitmap: &'static mut [u64], memory_size: usize) -> Self {
        let max_frames = bitmap.len() * 64;
        let requested_frames = memory_size / FRAME_SIZE;

        // Clamp to bitmap capacity rather than panicking – if the caller
        // passes a memory_size that exceeds what the bitmap can represent we
        // simply stop tracking frames beyond the bitmap limit.  Frames beyond
        // this point will never be allocated (total_frames caps them out).
        let total_frames = if requested_frames > max_frames {
            max_frames
        } else {
            requested_frames
        };

        for word in bitmap.iter_mut() {
            *word = 0;
        }

        Self {
            bitmap,
            total_frames,
        }
    }

    pub fn reserve(&mut self, frame: Frame) -> Result<(), MemoryFault> {
        self.mark_used(frame.index())
    }

    pub fn reserve_range(&mut self, start: usize, end: usize) -> Result<(), MemoryFault> {
        let sidx = Frame::new(start).index();
        let eidx = Frame::new(end).index();

        for idx in sidx..=eidx {
            self.mark_used(idx)?;
        }

        Ok(())
    }

    pub fn alloc(&mut self) -> Result<Frame, MemoryFault> {
        for idx in 0..self.total_frames {
            if !self.is_used(idx) {
                self.mark_used(idx)?;
                return Ok(Frame::from_index(idx));
            }
        }

        Err(MemoryFault::OutOfMemory)
    }

    pub fn free(&mut self, frame: Frame) -> Result<(), MemoryFault> {
        self.mark_free(frame.index())
    }

    fn is_used(&self, idx: usize) -> bool {
        let bidx = idx / 64;
        let bit = idx % 64;
        (self.bitmap[bidx] & (1 << bit)) != 0
    }

    fn mark_used(&mut self, idx: usize) -> Result<(), MemoryFault> {
        if idx >= self.total_frames {
            return Err(MemoryFault::FrameIndexOutOfBounds {
                idx,
                max: self.total_frames,
            });
        }

        if self.is_used(idx) {
            return Err(MemoryFault::DoubleAllocation { idx });
        }

        let bidx = idx / 64;
        let bit = idx % 64;

        self.bitmap[bidx] |= 1 << bit;

        Ok(())
    }

    fn mark_free(&mut self, idx: usize) -> Result<(), MemoryFault> {
        if idx >= self.total_frames {
            return Err(MemoryFault::FrameIndexOutOfBounds {
                idx,
                max: self.total_frames,
            });
        }

        if !self.is_used(idx) {
            return Err(MemoryFault::DoubleFree { idx });
        }

        let bidx = idx / 64;
        let bit = idx % 64;

        self.bitmap[bidx] &= !(1 << bit);

        Ok(())
    }
}
