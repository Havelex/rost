use crate::arch::x86_64::cpu::outb;

/// The PIT oscillator frequency in Hz.
const PIT_BASE_FREQUENCY: u32 = 1193182;

/// Commands for the Mode/Command register (0x43)
const SELECT_CHANNEL_0: u8 = 0b00_00_00_00;
const ACCESS_LOBYTE_HIBYTE: u8 = 0b00_11_00_00;
const MODE_SQUARE_WAVE: u8 = 0b00_00_01_10;

pub fn init(target_hz: u32) {
    // 1. Calculate the divisor
    let divisor = PIT_BASE_FREQUENCY / target_hz;

    // 2. Send the command byte
    // Channel 0, access lo/hi, square wave mode
    let command: u8 = SELECT_CHANNEL_0 | ACCESS_LOBYTE_HIBYTE | MODE_SQUARE_WAVE;

    unsafe {
        outb(0x43, command);

        // 3. Send the divisor (low byte then high byte) to Channel 0 (0x40)
        outb(0x40, (divisor & 0xFF) as u8);
        outb(0x40, ((divisor >> 8) & 0xFF) as u8);
    }

    crate::println!("PIT initialized at {}Hz", target_hz);
}
