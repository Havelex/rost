use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use crate::arch::x86_64::msr::{read, write};

// ── IA32_APIC_BASE MSR ────────────────────────────────────────────────────────
const IA32_APIC_BASE_MSR: u32 = 0x1B;
const IA32_APIC_BASE_MSR_ENABLE: u64 = 1 << 11;
const IA32_APIC_BASE_MSR_X2APIC: u64 = 1 << 10;

// ── x2APIC MSR addresses ──────────────────────────────────────────────────────
const X2APIC_ID: u32 = 0x802;
const X2APIC_TPR: u32 = 0x808;
const X2APIC_EOI: u32 = 0x80B;
const X2APIC_SIVR: u32 = 0x80F;
const X2APIC_ESR: u32 = 0x828;
const X2APIC_LVT_ERR: u32 = 0x837;
const X2APIC_LINT0: u32 = 0x835; // xAPIC offset 0x350 → MSR 0x800 + 0x35
const X2APIC_LINT1: u32 = 0x836; // xAPIC offset 0x360 → MSR 0x800 + 0x36
const X2APIC_SIVR_ENABLE: u64 = 1 << 8;

// ── xAPIC physical base and MMIO register offsets ────────────────────────────
const XAPIC_PHYS_BASE: usize = 0xFEE0_0000;
const XAPIC_SIZE: usize = 0x1000;

const XAPIC_ID_OFF: usize = 0x020;
const XAPIC_TPR_OFF: usize = 0x080;
const XAPIC_EOI_OFF: usize = 0x0B0;
const XAPIC_SIVR_OFF: usize = 0x0F0;
const XAPIC_SIVR_ENABLE: u32 = 1 << 8;
const XAPIC_ESR_OFF: usize = 0x280;
const XAPIC_LINT0_OFF: usize = 0x350;
const XAPIC_LINT1_OFF: usize = 0x360;
const XAPIC_LVT_ERR_OFF: usize = 0x370;

// ── IOAPIC physical base and register offsets ─────────────────────────────────
const IOAPIC_PHYS_BASE: usize = 0xFEC0_0000;
const IOAPIC_SIZE: usize = 0x1000;
const IOAPIC_REGSEL_OFF: usize = 0x00;
const IOAPIC_IOWIN_OFF: usize = 0x10;
const IOAPIC_REDTBL_BASE: u32 = 0x10;

// ── LVT mask bit (same meaning for both x2APIC and xAPIC) ────────────────────
const APIC_LVT_MASK32: u32 = 1 << 16;
const APIC_LVT_MASK64: u64 = 1 << 16;

// ── Active-controller discriminant ───────────────────────────────────────────
const CTRL_PIC: u8 = 0;
const CTRL_X2APIC: u8 = 1;
const CTRL_XAPIC: u8 = 2;

static ACTIVE_CONTROLLER: AtomicU8 = AtomicU8::new(CTRL_PIC);

/// Virtual base address of the xAPIC MMIO region.  Zero means not mapped yet.
static XAPIC_BASE: AtomicUsize = AtomicUsize::new(0);

/// Virtual base address of the IOAPIC MMIO region.  Zero means not mapped yet.
static IOAPIC_BASE: AtomicUsize = AtomicUsize::new(0);

/// Mask for extracting the LAPIC ID byte from the x2APIC ID MSR.
const X2APIC_ID_MASK: u64 = 0xFF;
/// Bit position of the LAPIC ID field in the xAPIC ID register (bits [31:24]).
const XAPIC_ID_SHIFT: u32 = 24;

// ── CPUID helpers ─────────────────────────────────────────────────────────────

/// Returns `true` if the CPU advertises an on-chip APIC (CPUID.1:EDX[9]).
///
/// CPUID leaf 1 is guaranteed to exist on all x86_64 CPUs per the AMD64
/// and Intel 64 architecture specifications.
pub fn has_apic() -> bool {
    let res = unsafe { core::arch::x86_64::__cpuid(1) };
    (res.edx & (1 << 9)) != 0
}

/// Returns `true` if the CPU supports x2APIC mode (CPUID.1:ECX[21]).
///
/// CPUID leaf 1 is guaranteed to exist on all x86_64 CPUs per the AMD64
/// and Intel 64 architecture specifications.
pub fn has_x2apic() -> bool {
    let res = unsafe { core::arch::x86_64::__cpuid(1) };
    (res.ecx & (1 << 21)) != 0
}

// ── xAPIC MMIO accessors ──────────────────────────────────────────────────────

/// Read a 32-bit xAPIC register at `base + offset`.
///
/// # Safety
/// `base` must be the valid virtual address of the xAPIC MMIO region.
unsafe fn xapic_read(base: usize, offset: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + offset) as *const u32) }
}

/// Write a 32-bit xAPIC register at `base + offset`.
///
/// # Safety
/// `base` must be the valid virtual address of the xAPIC MMIO region.
unsafe fn xapic_write(base: usize, offset: usize, value: u32) {
    unsafe { core::ptr::write_volatile((base + offset) as *mut u32, value) }
}

// ── IOAPIC MMIO accessors ─────────────────────────────────────────────────────

/// Read an IOAPIC indirect register.
///
/// # Safety
/// `base` must be the valid virtual address of the IOAPIC MMIO region.
#[allow(dead_code)]
unsafe fn ioapic_read(base: usize, reg: u32) -> u32 {
    unsafe {
        core::ptr::write_volatile((base + IOAPIC_REGSEL_OFF) as *mut u32, reg);
        core::ptr::read_volatile((base + IOAPIC_IOWIN_OFF) as *const u32)
    }
}

/// Write an IOAPIC indirect register.
///
/// # Safety
/// `base` must be the valid virtual address of the IOAPIC MMIO region.
unsafe fn ioapic_write(base: usize, reg: u32, value: u32) {
    unsafe {
        core::ptr::write_volatile((base + IOAPIC_REGSEL_OFF) as *mut u32, reg);
        core::ptr::write_volatile((base + IOAPIC_IOWIN_OFF) as *mut u32, value);
    }
}

// ── x2APIC initialization ─────────────────────────────────────────────────────

/// Configure x2APIC registers after enabling x2APIC mode in `IA32_APIC_BASE`.
///
/// # Safety
/// Must only be called while x2APIC mode is active in `IA32_APIC_BASE_MSR`.
unsafe fn init_x2apic_registers() {
    unsafe {
        // Allow all interrupt priorities.
        write(X2APIC_TPR, 0);

        // Clear the error status register (must write twice to reset).
        write(X2APIC_ESR, 0);
        write(X2APIC_ESR, 0);

        // Enable the APIC software-enable bit; use 0xFF as the spurious vector.
        write(X2APIC_SIVR, X2APIC_SIVR_ENABLE | 0xFF);

        // Mask local interrupt lines and the LVT error entry.
        write(X2APIC_LINT0, APIC_LVT_MASK64);
        write(X2APIC_LINT1, APIC_LVT_MASK64);
        write(X2APIC_LVT_ERR, APIC_LVT_MASK64);

        // Clear ESR again after LVT setup to acknowledge any errors.
        write(X2APIC_ESR, 0);
        write(X2APIC_ESR, 0);
    }
}

// ── xAPIC initialization ──────────────────────────────────────────────────────

/// Configure xAPIC MMIO registers.
///
/// # Safety
/// `base` must be the valid virtual address of the xAPIC MMIO region and the
/// global-enable bit in `IA32_APIC_BASE_MSR` must already be set.
unsafe fn init_xapic_registers(base: usize) {
    unsafe {
        // Allow all interrupt priorities.
        xapic_write(base, XAPIC_TPR_OFF, 0);

        // Clear the error status register (must write twice to reset).
        xapic_write(base, XAPIC_ESR_OFF, 0);
        xapic_write(base, XAPIC_ESR_OFF, 0);

        // Enable the APIC software-enable bit; use 0xFF as the spurious vector.
        xapic_write(base, XAPIC_SIVR_OFF, XAPIC_SIVR_ENABLE | 0xFF);

        // Mask local interrupt lines and the LVT error entry.
        xapic_write(base, XAPIC_LINT0_OFF, APIC_LVT_MASK32);
        xapic_write(base, XAPIC_LINT1_OFF, APIC_LVT_MASK32);
        xapic_write(base, XAPIC_LVT_ERR_OFF, APIC_LVT_MASK32);
    }
}

// ── IOAPIC initialization ─────────────────────────────────────────────────────

/// Program a single IOAPIC redirection-table entry.
///
/// Delivers `vector` via fixed (edge-triggered, active-high) mode to `dest`.
///
/// # Safety
/// `base` must be the valid virtual address of the IOAPIC MMIO region.
unsafe fn ioapic_set_redir(base: usize, irq: u8, vector: u8, dest: u8) {
    let reg_lo = IOAPIC_REDTBL_BASE + (irq as u32) * 2;
    let reg_hi = reg_lo + 1;

    // High half: destination LAPIC ID in bits [63:56] of the entry
    // (bits [31:24] of the high DWORD).
    let high: u32 = (dest as u32) << 24;
    // Low half: vector, fixed delivery, physical destination, active-high,
    // edge-triggered, not masked.
    let low: u32 = vector as u32;

    log_debug!(
        "[apic] ioapic_set_redir: irq={} vector={:#04x} dest_lapic={} reg_lo={:#x} reg_hi={:#x}",
        irq, vector, dest, reg_lo, reg_hi
    );

    unsafe {
        // Write high half first, then low half (which unmasks the entry).
        ioapic_write(base, reg_hi, high);
        ioapic_write(base, reg_lo, low);

        let rb_lo = ioapic_read(base, reg_lo);
        let rb_hi = ioapic_read(base, reg_hi);
        log_debug!(
            "[apic] ioapic redir readback: lo={:#010x} hi={:#010x}",
            rb_lo, rb_hi
        );
    }
}

/// Configure the IOAPIC to route IRQ 0 (PIT timer) to `lapic_id` as vector 32.
///
/// # Safety
/// `base` must be the valid virtual address of the IOAPIC MMIO region.
unsafe fn init_ioapic(base: usize, lapic_id: u8) {
    log_info!("[apic] init_ioapic: base={:#018x} lapic_id={}", base, lapic_id);
    // IRQ 0 → vector 32 (0x20), delivered to the LAPIC identified by lapic_id.
    unsafe { ioapic_set_redir(base, 0, 0x20, lapic_id) }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Attempt to initialize APIC after paging is active.
///
/// Tries x2APIC → xAPIC → falls back silently to the PIC (which was already
/// initialized during early boot).  On any failure the function leaves the
/// hardware in a consistent state with the PIC still active.
pub fn try_init_apic() {
    if has_x2apic() {
        // ── x2APIC path (MSR-based, no MMIO needed) ───────────────────────
        let mut base = unsafe { read(IA32_APIC_BASE_MSR) };
        base |= IA32_APIC_BASE_MSR_ENABLE | IA32_APIC_BASE_MSR_X2APIC;
        unsafe {
            write(IA32_APIC_BASE_MSR, base);
            init_x2apic_registers();
        }

        // Disable the 8259 PIC — APIC takes over all IRQ delivery.
        crate::arch::x86_64::cpu::interrupts::pic::disable();

        // Update the active controller BEFORE programming the IOAPIC.
        // In QEMU (and on real hardware) a pending PIT pulse is injected into
        // the LAPIC the instant the unmasked IOAPIC redirection entry is
        // written.  If ACTIVE_CONTROLLER still says CTRL_PIC at that point,
        // send_eoi() sends the EOI to the 8259 instead of the LAPIC, leaving
        // LAPIC ISR[32] set permanently — no further vector-32 interrupts
        // can ever be delivered.
        ACTIVE_CONTROLLER.store(CTRL_X2APIC, Ordering::Release);

        // Try to bring up the IOAPIC for PIT-timer routing.
        log_info!("[apic] mapping IOAPIC...");
        match crate::arch::x86_64::memory::paging::map_mmio_region(
            IOAPIC_PHYS_BASE,
            IOAPIC_SIZE,
        ) {
            Ok(ioapic_virt) => {
                log_info!("[apic] IOAPIC mapped at virt={:#018x}", ioapic_virt);
                IOAPIC_BASE.store(ioapic_virt, Ordering::Release);
                // Read LAPIC ID from the x2APIC ID MSR (low 8 bits).
                let lapic_id = (unsafe { read(X2APIC_ID) } & X2APIC_ID_MASK) as u8;
                log_info!("[apic] x2APIC LAPIC ID: {}", lapic_id);
                unsafe { init_ioapic(ioapic_virt, lapic_id) };
            }
            Err(_) => {
                log_warn!("[apic] IOAPIC mapping failed; timer IRQ0 not routed");
            }
        }

        log_ok!("[apic] x2APIC initialized");
        log_info!("[apic] active controller: {}", active_controller());
        return;
    }

    if has_apic() {
        // ── xAPIC path (MMIO-based) ───────────────────────────────────────
        match crate::arch::x86_64::memory::paging::map_mmio_region(XAPIC_PHYS_BASE, XAPIC_SIZE) {
            Ok(virt) => {
                let base = unsafe { read(IA32_APIC_BASE_MSR) };
                log_info!(
                    "[apic] IA32_APIC_BASE_MSR: {:#018x} (x2apic={}, en={})",
                    base,
                    (base & IA32_APIC_BASE_MSR_X2APIC) != 0,
                    (base & IA32_APIC_BASE_MSR_ENABLE) != 0,
                );

                // Preserve the physical base address (bits [47:12]) and
                // reserved upper bits; clear the control bits [11:0].
                // Transition through the "disabled" state (EN=0, EXTD=0)
                // before enabling xAPIC mode (EN=1, EXTD=0).  This is
                // required by the Intel manual when moving from x2APIC to
                // xAPIC, and is a no-op when already in xAPIC or disabled
                // state.  It also handles the case where firmware (Limine)
                // left the APIC in x2APIC mode — without this step, MMIO
                // writes to SIVR/TPR are silently discarded and the LAPIC
                // never accepts external interrupts.
                let phys_base = base & 0xFFFF_FFFF_FFFF_F000u64;
                unsafe {
                    write(IA32_APIC_BASE_MSR, phys_base);                               // disabled
                    write(IA32_APIC_BASE_MSR, phys_base | IA32_APIC_BASE_MSR_ENABLE);  // xAPIC
                    init_xapic_registers(virt);
                }

                // Confirm the software-enable bit was written successfully.
                let sivr = unsafe { xapic_read(virt, XAPIC_SIVR_OFF) };
                log_info!(
                    "[apic] xAPIC SIVR readback: {:#010x} (sw_enabled={})",
                    sivr,
                    (sivr & XAPIC_SIVR_ENABLE) != 0,
                );

                XAPIC_BASE.store(virt, Ordering::Release);

                // Disable the 8259 PIC — APIC takes over all IRQ delivery.
                crate::arch::x86_64::cpu::interrupts::pic::disable();

                // Update the active controller BEFORE programming the IOAPIC
                // for the same reason as in the x2APIC path above.
                ACTIVE_CONTROLLER.store(CTRL_XAPIC, Ordering::Release);

                // Try to bring up the IOAPIC for PIT-timer routing.
                log_info!("[apic] mapping IOAPIC...");
                match crate::arch::x86_64::memory::paging::map_mmio_region(
                    IOAPIC_PHYS_BASE,
                    IOAPIC_SIZE,
                ) {
                    Ok(ioapic_virt) => {
                        log_info!("[apic] IOAPIC mapped at virt={:#018x}", ioapic_virt);
                        IOAPIC_BASE.store(ioapic_virt, Ordering::Release);
                        // Read xAPIC LAPIC ID from register 0x020, bits [31:24].
                        let lapic_id =
                            (unsafe { xapic_read(virt, XAPIC_ID_OFF) } >> XAPIC_ID_SHIFT) as u8;
                        log_info!("[apic] xAPIC LAPIC ID: {}", lapic_id);
                        unsafe { init_ioapic(ioapic_virt, lapic_id) };
                    }
                    Err(_) => {
                        log_warn!("[apic] IOAPIC mapping failed; timer IRQ0 not routed");
                    }
                }

                log_ok!("[apic] xAPIC initialized");
                log_info!("[apic] active controller: {}", active_controller());
            }
            Err(_) => {
                log_warn!("[apic] xAPIC MMIO mapping failed, staying on PIC");
            }
        }
        return;
    }

    // Neither APIC variant is available; the PIC remains active.
    log_info!("[apic] No APIC available, staying on PIC");
    log_info!("[apic] active controller: {}", active_controller());
}

/// Send an end-of-interrupt signal to the active interrupt controller.
///
/// The `irq` parameter is used only when the PIC is active (to decide whether
/// a cascade EOI is also needed).  APIC modes ignore it.
pub fn send_eoi(irq: u8) {
    match ACTIVE_CONTROLLER.load(Ordering::Acquire) {
        CTRL_X2APIC => unsafe { write(X2APIC_EOI, 0) },
        CTRL_XAPIC => {
            let base = XAPIC_BASE.load(Ordering::Acquire);
            unsafe { xapic_write(base, XAPIC_EOI_OFF, 0) };
        }
        _ => crate::arch::x86_64::cpu::interrupts::pic::send_eoi(irq),
    }
}

/// Return a short string describing the currently active interrupt controller.
#[allow(dead_code)]
pub fn active_controller() -> &'static str {
    match ACTIVE_CONTROLLER.load(Ordering::Acquire) {
        CTRL_X2APIC => "x2APIC",
        CTRL_XAPIC => "xAPIC",
        _ => "PIC",
    }
}
