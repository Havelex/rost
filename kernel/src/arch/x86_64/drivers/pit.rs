use crate::{
    arch::x86_64::{asm::outb, cpu::interrupts::pic},
    error::Result,
};

/// The PIT oscillator frequency in Hz.
const PIT_BASE_FREQUENCY: u32 = 1_193_182;

const COMMAND_PORT: u16 = 0x43;
const CHANNEL_0_PORT: u16 = 0x40;

const SELECT_CHANNEL_0: u8 = 0b00_00_00_00;
const ACCESS_LOBYTE_HIBYTE: u8 = 0b00_11_00_00;
const MODE_SQUARE_WAVE: u8 = 0b00_00_01_10;
const TARGET_HZ: u32 = 100;

pub fn init() -> Result<()> {
    let divisor = PIT_BASE_FREQUENCY / TARGET_HZ;
    let low_byte = (divisor & 0xFF) as u8;
    let high_byte = ((divisor >> 8) & 0xFF) as u8;

    let command: u8 = SELECT_CHANNEL_0 | ACCESS_LOBYTE_HIBYTE | MODE_SQUARE_WAVE;

    unsafe {
        outb(COMMAND_PORT, command);
        outb(CHANNEL_0_PORT, low_byte);
        outb(CHANNEL_0_PORT, high_byte);
    }

    // The PIT counter is now fully programmed.  Unmask IRQ0 so that timer
    // interrupts begin firing from a known, configured state.  This call is
    // deliberately placed here rather than in pic::init() / interrupts::init()
    // to prevent the firmware-default (typically ~18 Hz) timer from firing
    // during early boot before the kernel is ready.
    //
    // When the APIC is active the PIC is already disabled at this point, so
    // this write is harmless — timer interrupts arrive via the IOAPIC entry
    // that was programmed during try_init_apic().
    pic::clear_mask(pic::IRQ_PIT_TIMER);

    Ok(())
}
