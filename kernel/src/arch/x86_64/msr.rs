pub unsafe fn read(msr: u32) -> u64 {
    let (high, low): (u32, u32);
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("edx") high,
            out("eax") low,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

pub unsafe fn write(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("edx") high,
            in("eax") low,
        );
    }
}
