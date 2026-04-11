use crate::memory::regions::MemoryRegionKind;

impl From<u64> for MemoryRegionKind {
    fn from(v: u64) -> Self {
        match v {
            0 => Self::Usable,
            1 => Self::Reserved,
            2 => Self::AcpiReclaimable,
            3 => Self::AcpiNvs,
            4 => Self::BadMemory,
            5 => Self::BootloaderReclaimable,
            6 => Self::KernelAndModules,
            7 => Self::Framebuffer,
            x => Self::Unknown(x),
        }
    }
}
