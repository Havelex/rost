use crate::{
    arch::x86_64::cpu::{inb, outb},
    error::Result,
};

// Port addresses are 16-bit on x86
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

pub unsafe fn remap_pic(offset1: u8, offset2: u8) {
    // ICW1: Start initialization
    outb(PIC1_COMMAND, 0x11);
    io_wait();
    outb(PIC2_COMMAND, 0x11);
    io_wait();

    // ICW2: Vector offsets
    outb(PIC1_DATA, offset1);
    outb(PIC2_DATA, offset2);

    // ICW3: Cascade
    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);

    // ICW4: 8086 mode
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);

    outb(PIC1_DATA, 0xFF);
    outb(PIC2_DATA, 0xFF);
}

fn io_wait() {
    outb(0x80, 0);
}

unsafe fn unmask_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq_bit = if irq < 8 { irq } else { irq - 8 };

    // Read current mask, clear the bit for our IRQ, and write it back
    // 0 = Enabled, 1 = Masked
    let mask = inb(port) & !(1 << irq_bit);
    outb(port, mask);
}

unsafe fn disable_pic() {
    outb(PIC1_DATA, 0xFF);
    outb(PIC2_DATA, 0xFF);
}

pub fn init() -> Result<()> {
    unsafe {
        // 1. Remap the PIC vectors to 32 and 40
        remap_pic(32, 40);
        unmask_irq(0);
        unmask_irq(1);
        unmask_irq(2);
    }
    Ok(())
}
