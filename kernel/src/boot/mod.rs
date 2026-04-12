mod boot_info;
mod limine_helpers;

pub use boot_info::*;

pub fn init() -> BootInfo {
    BootInfo::new();

    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
    }
}
