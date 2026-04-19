use crate::{arch::x86_64::asm::outb, error::Result};

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
    log_info!(
        "[pit] programming: base={}Hz target={}Hz divisor={} command={:#04x}",
        PIT_BASE_FREQUENCY,
        TARGET_HZ,
        divisor,
        command
    );
    log_info!(
        "[pit] writing ports: CH0={:#04x} low={:#04x} high={:#04x}",
        CHANNEL_0_PORT, low_byte, high_byte
    );
    unsafe {
        outb(COMMAND_PORT, command);
        outb(CHANNEL_0_PORT, low_byte);
        outb(CHANNEL_0_PORT, high_byte);
    }
    log_ok!("[pit] initialized at {}Hz", TARGET_HZ);
    Ok(())
}
