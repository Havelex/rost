//! Allocator for memory frames in a simple kernel.
//!
//! A simple frame allocator that uses a bitmap to track allocated and free frames of memory.
//! This is a very basic implementation and is not optimized for performance or fragmentation.
//! It is intended for use in a simple kernel where memory management is not a primary concern,
//! and is designed to be easy to understand and modify for educational purposes.

use crate::panic::KernelFault;

/// Size of a memory frame in bytes.  This is typically the same as the page size
pub const FRAME_SIZE: usize = 0x1000;

/// Enum representing various memory allocation faults that can occur in the frame allocator.
#[allow(dead_code)]
#[derive(Debug)]
pub enum MemoryFault {
    /// A frame index exceeded the allocator's tracked range.
    FrameIndexOutOfBounds { idx: usize, max: usize },
    /// Attempted to allocate a frame that was already allocated.
    DoubleAllocation { idx: usize },
    /// Attempted to free a frame that was already free.
    DoubleFree { idx: usize },
    /// No free frames remain in the allocator.
    OutOfMemory,
    /// The frame allocator has not been initialised yet.
    NoAllocator,
}

impl From<MemoryFault> for KernelFault {
    fn from(err: MemoryFault) -> Self {
        KernelFault::Memory(err)
    }
}

/// Represents a memory frame, which is a contiguous block of memory of size FRAME_SIZE.
#[derive(Clone, Copy, Debug)]
pub struct Frame(usize);

impl Frame {
    /// Creates a new Frame from a given address, aligning it down to the nearest frame boundary.
    ///
    /// # Parameters
    /// - `addr`: The address to create the frame from.
    ///
    /// # Returns
    /// A new `Frame` instance representing the frame containing the given address.
    pub const fn new(addr: usize) -> Self {
        Self(addr & !(FRAME_SIZE - 1))
    }

    /// Creates a Frame from a given frame index.
    ///
    /// # Parameters
    /// - `idx`: The index of the frame to create.
    ///
    /// # Returns
    /// A new `Frame` instance representing the frame at the specified index.
    pub const fn from_index(idx: usize) -> Self {
        Self(idx * FRAME_SIZE)
    }

    /// Returns the starting address of the frame.
    ///
    /// # Returns
    /// The starting address of the frame as a `usize`.
    pub const fn addr(self) -> usize {
        self.0
    }

    /// Returns the index of the frame.
    ///
    /// # Returns
    /// The index of the frame as a `usize`.
    pub const fn index(self) -> usize {
        self.0 / FRAME_SIZE
    }
}

/// A bitmap-based physical frame allocator.
///
/// Manages a contiguous range of 4 KiB physical frames.  Each bit in the
/// backing bitmap represents one frame: `0` = free, `1` = allocated/reserved.
pub struct FrameAllocator {
    bitmap: &'static mut [u64],
    total_frames: usize,
}

impl FrameAllocator {
    /// Create a new allocator backed by `bitmap`.
    ///
    /// All bitmap words are zeroed (all frames initially free).  If
    /// `memory_size` would require more frames than the bitmap can represent
    /// the frame count is silently clamped to the bitmap capacity.
    ///
    /// # Parameters
    /// - `bitmap`: A mutable reference to the static array used as the backing bitmap.
    /// - `memory_size`: Total physical memory size in bytes; determines how many frames to manage.
    ///
    /// # Returns
    /// A new [`FrameAllocator`] ready for use.
    ///
    /// # Safety
    /// The caller must ensure `bitmap` lives for `'static` and is not shared.
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

    /// Reserve a single frame, marking it as allocated.
    ///
    /// # Parameters
    /// - `frame`: The frame to reserve.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(MemoryFault::DoubleAllocation)` if the frame was already allocated.
    #[allow(dead_code)]
    pub fn reserve(&mut self, frame: Frame) -> Result<(), MemoryFault> {
        self.mark_used(frame.index())
    }

    /// Reserve every frame in the inclusive physical address range `[start, end]`.
    ///
    /// Addresses are aligned down to frame boundaries before marking.
    ///
    /// # Parameters
    /// - `start`: Physical start address of the range to reserve.
    /// - `end`: Physical end address (inclusive) of the range to reserve.
    ///
    /// # Returns
    /// - `Ok(())` if all frames were successfully reserved.
    /// - `Err(MemoryFault)` if any frame could not be reserved.
    pub fn reserve_range(&mut self, start: usize, end: usize) -> Result<(), MemoryFault> {
        let sidx = Frame::new(start).index();
        let eidx = Frame::new(end).index();

        for idx in sidx..=eidx {
            self.mark_used(idx)?;
        }

        Ok(())
    }

    /// Allocate the lowest free physical frame.
    ///
    /// # Returns
    /// - `Ok(Frame)` containing the allocated frame's physical address.
    /// - `Err(MemoryFault::OutOfMemory)` if no free frames remain.
    pub fn alloc(&mut self) -> Result<Frame, MemoryFault> {
        for idx in 0..self.total_frames {
            if !self.is_used(idx) {
                self.mark_used(idx)?;
                return Ok(Frame::from_index(idx));
            }
        }

        Err(MemoryFault::OutOfMemory)
    }

    /// Free a previously allocated frame.
    ///
    /// # Parameters
    /// - `frame`: The frame to free.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(MemoryFault::DoubleFree)` if the frame was already free.
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
