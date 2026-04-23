//! Kernel boot entry point.
//!
//! This module collects all Limine bootloader responses and packages them into
//! a [`BootInfo`] struct that is passed to [`crate::init`].

mod boot_info;
mod limine_helpers;

pub use boot_info::*;

/// Collect boot-time information from the Limine bootloader.
///
/// This function must be called once, as early as possible, by the
/// architecture-specific entry point before calling [`crate::init`].
///
/// # Returns
/// A [`BootInfo`] populated from all active Limine request responses.
pub fn init() -> BootInfo {
    BootInfo::new()
}
