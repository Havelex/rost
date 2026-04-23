//! x86_64 hardware driver initialisation.
//!
//! Initialises the PIT timer and unmasks the PS/2 keyboard IRQ line.
//! This module is called from [`crate::arch::x86_64::X86_64::init_drivers`].

use crate::{error::Result, init_step};

pub mod hpet;
pub mod pit;

/// Initialise all x86_64 hardware drivers.
///
/// Currently initialises the PIT at 100 Hz and unmasks IRQ 1 (PS/2 keyboard).
///
/// # Returns
/// - `Ok(())` if all drivers initialised successfully.
/// - `Err(KernelError)` if a driver initialisation step fails.
pub fn init() -> Result<()> {
    log_info!(
        "[drivers] active interrupt controller: {}",
        crate::arch::x86_64::cpu::interrupts::apic::active_controller()
    );
    init_step("Initializing PIT", "PIT initialized", pit::init)?;

    // Unmask IRQ1 (PS/2 keyboard) so that key-press scancodes are delivered.
    // When the APIC is active the PIC is already disabled, so this call is
    // harmless — keyboard interrupts arrive via the IOAPIC entry programmed
    // during try_init_apic().
    crate::arch::x86_64::cpu::interrupts::pic::clear_mask(
        crate::arch::x86_64::cpu::interrupts::pic::IRQ_KEYBOARD,
    );

    Ok(())
}
