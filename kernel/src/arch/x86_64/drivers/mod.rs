use crate::{error::Result, init_step};

pub mod hpet;
pub mod pit;

pub fn init() -> Result<()> {
    init_step("Initializing PIT...", pit::init)?;
    Ok(())
}
