use crate::{arch::x86_64::cpu::outb, error::Result};

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

    // let command: u8 = SELECT_CHANNEL_0 | ACCESS_LOBYTE_HIBYTE | MODE_SQUARE_WAVE;
    let command: u8 = 0x36;

    outb(COMMAND_PORT, command);

    outb(0x80, 0);
    // 3. Send the divisor (low byte then high byte) to Channel 0 (0x40)
    outb(CHANNEL_0_PORT, low_byte);
    outb(0x80, 0);
    outb(CHANNEL_0_PORT, high_byte);

    Ok(())
}
