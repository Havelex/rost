use core::arch::asm;

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",  // Intel syntax: out DEST (port), SRC (value)
            in("dx") port,
            in("al") val,
            options(nostack, preserves_flags)
        );
    }
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let res: u8;
    unsafe {
        asm!(
            "in al, dx",   // Intel syntax: in DEST (value), SRC (port)
            in("dx") port,
            out("al") res,
            options(nostack, preserves_flags)
        );
    }
    res
}
