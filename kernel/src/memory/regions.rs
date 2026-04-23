//! Kernel-internal memory region types.
//!
//! Converts the raw Limine memory-map entries ([`MemMapInfo`](crate::boot::MemMapInfo))
//! into a fixed-size [`MemMap`] using the [`MemoryRegionKind`] enum for type
//! classification.

use crate::boot::{MAX_REGIONS, MemMapInfo, MemoryRegionInfo};

/// Classification of a physical memory region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRegionKind {
    /// General-purpose RAM available for kernel use.
    Usable,
    /// Reserved by firmware; must not be touched.
    Reserved,
    /// ACPI tables — may be reclaimed after ACPI initialisation.
    AcpiReclaimable,
    /// ACPI Non-Volatile Storage area.
    AcpiNvs,
    /// Memory reported as defective by the firmware.
    BadMemory,
    /// Memory used by the bootloader; may be reclaimed after boot.
    BootloaderReclaimable,
    /// Physical frames occupied by the kernel image and modules.
    KernelAndModules,
    /// Physical frames used by the bootloader framebuffer.
    Framebuffer,
    /// An unrecognised Limine memory type; the raw value is preserved.
    Unknown(u64),
}

/// A single physical memory region descriptor.
#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    /// Physical base address of the region.
    pub base: usize,
    /// Length of the region in bytes.
    pub length: usize,
    /// Categorised memory type for this region.
    pub kind: MemoryRegionKind,
}

impl From<MemoryRegionInfo> for MemoryRegion {
    fn from(value: MemoryRegionInfo) -> Self {
        Self {
            base: value.base,
            length: value.length,
            kind: value.kind.into(),
        }
    }
}

/// A fixed-capacity collection of physical memory regions.
///
/// Holds up to [`MAX_REGIONS`] entries converted from the Limine memory map.
/// Used by [`phys::init`](crate::memory::phys::init) to set up the frame allocator.
#[derive(Copy, Clone)]
pub struct MemMap {
    /// Array of region descriptors (valid entries are `regions[..count]`).
    pub regions: [MemoryRegion; MAX_REGIONS],
    /// Number of valid entries in `regions`.
    pub count: usize,
    /// Highest physical address seen across all regions (total address space size).
    pub total_mem_size: usize,
}

impl From<MemMapInfo> for MemMap {
    fn from(value: MemMapInfo) -> Self {
        let mut regions = [MemoryRegion {
            base: 0,
            length: 0,
            kind: MemoryRegionKind::Reserved,
        }; MAX_REGIONS];

        let mut count = 0;
        let mut total_mem_size = 0;

        for (i, region) in value.regions.iter().enumerate() {
            if i >= MAX_REGIONS {
                break;
            }

            let converted = MemoryRegion::from(*region);

            total_mem_size = total_mem_size.max(converted.base.saturating_add(converted.length));

            regions[i] = converted;
            count += 1;
        }

        Self {
            regions,
            count,
            total_mem_size,
        }
    }
}
