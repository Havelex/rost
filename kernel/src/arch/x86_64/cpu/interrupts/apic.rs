use crate::arch::x86_64::msr::{read, write};

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const IA32_APIC_BASE_MSR_ENABLE: u64 = 1 << 11;
const IA32_APIC_BASE_MSR_X2APIC: u64 = 1 << 10;
const APIC_LVT_MASK: u64 = 1 << 16;

const X2APIC_SIVR: u32 = 0x80F; // Spurious Interrupt Vector Register
const X2APIC_SIVR_ENABLE: u64 = 1 << 8;
const X2APIC_TPR: u32 = 0x808;
const X2APIC_LINT0: u32 = 0x82F;
const X2APIC_LINT1: u32 = 0x830;
const X2APIC_EOI: u32 = 0x80B;

// In init_x2apic_registers:

pub unsafe fn init_apic() {
    let mut base = unsafe { read(IA32_APIC_BASE_MSR) };

    // Ensure Global Enable is set first
    base |= IA32_APIC_BASE_MSR_ENABLE;
    unsafe { write(IA32_APIC_BASE_MSR, base) };

    if has_x2apic() {
        base |= IA32_APIC_BASE_MSR_X2APIC;
        unsafe {
            write(IA32_APIC_BASE_MSR, base);
            init_x2apic_registers();
        }
    } else {
        unsafe {
            write(IA32_APIC_BASE_MSR, base);
        }
        // Fallback to MMIO if x2APIC is not supported
        // (This would require paging/mapping FEE00000)
    }
}

pub fn has_apic() -> bool {
    let res = core::arch::x86_64::__cpuid(1);
    (res.edx & (1 << 9)) != 0 // EDX bit 9 is APIC
}

pub fn has_x2apic() -> bool {
    let res = core::arch::x86_64::__cpuid(1);
    (res.ecx & (1 << 21)) != 0 // ECX bit 21 is x2APIC
}

unsafe fn init_x2apic_registers() {
    // 1. Allow all interrupts
    unsafe {
        write(X2APIC_TPR, 0);
    }

    // 2. Set spurious vector and enable software bit
    let sivr = X2APIC_SIVR_ENABLE | 0xFF;
    unsafe {
        write(X2APIC_SIVR, sivr);
        write(X2APIC_LINT0, APIC_LVT_MASK);
        write(X2APIC_LINT1, APIC_LVT_MASK);
    }
}

pub fn send_eoi() {
    unsafe {
        write(X2APIC_EOI, 0);
    }
}
