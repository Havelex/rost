use crate::{error::Result, init_step};

pub mod hpet;
pub mod pit;

pub fn init() -> Result<()> {
    log_info!(
        "[drivers] active interrupt controller: {}",
        crate::arch::x86_64::cpu::interrupts::apic::active_controller()
    );
    init_step("Initializing PIT...", pit::init)?;
    Ok(())
}
