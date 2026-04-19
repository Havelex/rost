use crate::{error::Result, init_step};

pub mod hpet;
pub mod pit;

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
