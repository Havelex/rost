use crate::{
    arch::{Arch, Architecture},
    cpu::Cpu,
    time::{get_ticks, increment_ticks},
};

pub fn handle_hardware_interrupt(irq: u8) {
    match irq {
        0 => {
            println!("Reached increment");
            increment_ticks();
            if get_ticks() % 100 == 0 {
                print!(".")
            }
        }
        1 => { /* Keyboard logic */ }
        _ => {
            crate::println!("Received hardware IRQ: {}", irq);
        }
    }

    <Arch as Architecture>::Cpu::send_eoi(irq);
}
