use crate::boot::{MAX_REGIONS, MemMapInfo, MemoryRegionInfo};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    BootloaderReclaimable,
    KernelAndModules,
    Framebuffer,
    Unknown(u64),
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    pub base: usize,
    pub length: usize,
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

#[derive(Copy, Clone)]
pub struct MemMap {
    pub regions: [MemoryRegion; MAX_REGIONS],
    pub count: usize,
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

            total_mem_size = total_mem_size.max(converted.base + converted.length);

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
