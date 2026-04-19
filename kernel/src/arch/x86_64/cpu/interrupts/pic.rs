use crate::{
    arch::x86_64::asm::{inb, outb},
    error::Result,
};

// Port addresses
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = PIC1_COMMAND + 1;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = PIC2_COMMAND + 1;

const PIC_EOI: u8 = 0x20;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

const CASCADE_IRQ: u8 = 2;
pub const IRQ_PIT_TIMER: u8 = 0;
pub const IRQ_KEYBOARD: u8 = 1;
pub const IRQ_CASCADE: u8 = 2;

const PIC_READ_IRR: u8 = 0x0a;
const PIC_READ_ISR: u8 = 0x0b;

/// Wait for a small amount of time for IO ports to settle
fn io_wait() {
    unsafe { outb(0x80, 0) };
}

/// Remaps the PIC offsets so they don't conflict with CPU exceptions
unsafe fn remap_pic(offset1: u8, offset2: u8) {
    unsafe {
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();

        outb(PIC1_DATA, offset1);
        io_wait();
        outb(PIC2_DATA, offset2);
        io_wait();

        outb(PIC1_DATA, 1 << CASCADE_IRQ);
        io_wait();
        outb(PIC2_DATA, 2);
        io_wait();

        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        outb(PIC1_DATA, 0);
        outb(PIC2_DATA, 0);
    }
}

/// Mask (disable) a specific IRQ line
pub fn set_mask(mut irq_line: u8) {
    let port = if irq_line < 8 {
        PIC1_DATA
    } else {
        irq_line -= 8;
        PIC2_DATA
    };
    unsafe {
        let value = inb(port) | (1 << irq_line);
        outb(port, value);
    }
}

/// Unmask (enable) a specific IRQ line
pub fn clear_mask(mut irq_line: u8) {
    let port = if irq_line < 8 {
        PIC1_DATA
    } else {
        irq_line -= 8;
        PIC2_DATA
    };
    unsafe {
        let value = inb(port) & !(1 << irq_line);
        outb(port, value);
    }
}

pub fn init() -> Result<()> {
    unsafe {
        remap_pic(0x20, 0x28);
        // Mask all IRQ lines after remap; callers must explicitly unmask what they need.
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }
    Ok(())
}

pub fn send_eoi(irq: u8) {
    if irq >= 8 {
        unsafe { outb(PIC2_COMMAND, PIC_EOI) };
    }
    unsafe { outb(PIC1_COMMAND, PIC_EOI) };
}

// ... rest of your register reading functions remain the same ...

unsafe fn pic_get_irq_reg(ocw3: u8) -> u16 {
    unsafe {
        outb(PIC1_COMMAND, ocw3);
    }
    unsafe {
        outb(PIC2_COMMAND, ocw3);
    }

    ((unsafe { inb(PIC2_COMMAND) } as u16) << 8) | (unsafe { inb(PIC1_COMMAND) } as u16)
}

#[inline]
pub unsafe fn pic_get_irr() -> u16 {
    unsafe { pic_get_irq_reg(PIC_READ_IRR) }
}

#[inline]
pub unsafe fn pic_get_isr() -> u16 {
    unsafe { pic_get_irq_reg(PIC_READ_ISR) }
}

pub fn disable() {
    unsafe {
        // Writing 0xFF to the data ports masks all interrupts on the 8259 PIC
        crate::arch::x86_64::asm::outb(0x21, 0xFF);
        crate::arch::x86_64::asm::outb(0xA1, 0xFF);
    }
}
