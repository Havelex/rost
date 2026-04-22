//! x86_64 Model-Specific Register (MSR) access.
//!
//! Exposes raw `RDMSR` / `WRMSR` wrappers used by the APIC and other
//! subsystems that communicate with the CPU through MSRs.

/// Read a 64-bit value from the Model-Specific Register identified by `msr`.
///
/// # Parameters
/// - `msr`: The 32-bit MSR address (e.g. `0x1B` for `IA32_APIC_BASE`).
///
/// # Returns
/// The 64-bit value currently stored in the MSR.
///
/// # Safety
/// The caller must ensure that:
/// - `msr` is a valid MSR address supported by the current CPU.
/// - Reading the MSR is safe in the current privilege level and execution context.
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

/// Write a 64-bit value to the Model-Specific Register identified by `msr`.
///
/// # Parameters
/// - `msr`: The 32-bit MSR address to write.
/// - `value`: The 64-bit value to store in the MSR.
///
/// # Safety
/// The caller must ensure that:
/// - `msr` is a valid, writable MSR address supported by the current CPU.
/// - Writing `value` to `msr` is safe and will not destabilize the system
///   (e.g. writing an invalid base address to `IA32_APIC_BASE` can cause
///   immediate machine check exceptions).
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
