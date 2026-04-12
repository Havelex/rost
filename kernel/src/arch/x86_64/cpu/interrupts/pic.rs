use crate::arch::x86_64::cpu::{inb, outb};

// Port addresses are 16-bit on x86
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

pub unsafe fn remap_pic(offset1: u8, offset2: u8) {
    // Save masks
    let a1 = inb(PIC1_DATA);
    let a2 = inb(PIC2_DATA);

    // ICW1: Start initialization
    outb(PIC1_COMMAND, 0x11);
    outb(PIC2_COMMAND, 0x11);

    // ICW2: Vector offsets (0-31 are reserved for exceptions, so offsets start at 32)
    outb(PIC1_DATA, offset1);
    outb(PIC2_DATA, offset2);

    // ICW3: Cascade identity
    outb(PIC1_DATA, 0x04); // Master has slave at IRQ2
    outb(PIC2_DATA, 0x02); // Slave identity is 2

    // ICW4: 8086 mode
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);

    // Restore masks
    outb(PIC1_DATA, a1);
    outb(PIC2_DATA, a2);
}

pub unsafe fn disable_pic() {
    outb(PIC1_DATA, 0xFF);
    outb(PIC2_DATA, 0xFF);
}
