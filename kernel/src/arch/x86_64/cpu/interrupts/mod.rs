use crate::{
    arch::x86_64::cpu::interrupts::apic::{has_apic, init_apic},
    cpu::interrupts::{
        GenericInterrupt, InterruptKind, exceptions::ExceptionType, handle_interrupt,
    },
    error::Result,
    init_step,
};

static mut SEND_EOI: fn(irq: u8) = pic::send_eoi;

pub mod apic;
mod idt;
pub mod pic;

#[repr(C)]
pub struct InterruptContext {
    // 15 GP Registers (Capture starts at r15)
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    // Directly follows rax on the stack
    pub vector: u64,
    pub error_code: u64,

    // CPU pushed
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn x86_64_interrupt_handler(ctx: *const InterruptContext) {
    let ctx = unsafe { &*ctx };

    let kind = if ctx.vector < 32 {
        InterruptKind::Exception(match ctx.vector {
            0 => ExceptionType::DivideByZero,
            3 => ExceptionType::Breakpoint,
            8 => ExceptionType::DoubleFault,
            13 => ExceptionType::GeneralProtectionFault(ctx.error_code), // Map GPF
            14 => {
                let addr: u64;
                unsafe {
                    core::arch::asm!("mov {}, cr2", out(reg) addr);
                }
                ExceptionType::PageFault {
                    addr,
                    error_code: ctx.error_code,
                }
            }
            v => ExceptionType::Unknown(v), // Actually unknown vectors
        })
    } else {
        InterruptKind::Hardware((ctx.vector - 32) as u8)
    };

    // Only dump registers for crashes, not for every timer tick!
    if ctx.vector < 32 && ctx.vector != 3 {
        dump_registers(ctx);
    }

    handle_interrupt(GenericInterrupt { rip: ctx.rip, kind });
}

fn dump_page_fault_details(error_code: u64) {
    let present = if error_code & (1 << 0) != 0 {
        "Protection Violation"
    } else {
        "Page Not Present"
    };
    let write = if error_code & (1 << 1) != 0 {
        "Write"
    } else {
        "Read"
    };
    let user = if error_code & (1 << 2) != 0 {
        "User"
    } else {
        "Kernel"
    };
    let fetch = if error_code & (1 << 4) != 0 {
        "Instruction Fetch"
    } else {
        "Data Access"
    };

    crate::println!(
        "PAGE FAULT TYPE: {} during {} by {} ({})",
        present,
        write,
        user,
        fetch
    );
}

fn dump_registers(ctx: &InterruptContext) {
    crate::println!("\n--- [ KERNEL PANIC ] ---");

    // If it's a Page Fault (14), print the CR2 register and decoded error
    if ctx.vector == 14 {
        let cr2: u64;
        unsafe {
            core::arch::asm!("mov {}, cr2", out(reg) cr2);
        }
        crate::println!("FAULT ADDRESS: {:#018x}", cr2);
        dump_page_fault_details(ctx.error_code);
    }

    crate::println!("VECTOR: {}  ERROR CODE: {:#x}", ctx.vector, ctx.error_code);
    crate::println!("RIP: {:#018x}  RSP: {:#018x}", ctx.rip, ctx.rsp);
    crate::println!("RAX: {:#018x}  RBX: {:#018x}", ctx.rax, ctx.rbx);
    crate::println!("RFLAGS: {:#018b}", ctx.rflags);
    crate::println!("------------------------\n");
}

pub fn init() -> Result<()> {
    init_step("Initializing IDT...", idt::init)?;
    log_info!("Testing breakpoint exception");
    unsafe {
        core::arch::asm!("int3");
    }
    log_ok!("Successfully returned from breakpoint.");

    init_step("Initializing Interrupt Controller...", || {
        if has_apic() {
            unsafe {
                SEND_EOI = |_: u8| {
                    apic::send_eoi();
                };
            }
            log_info!("APIC detected. Initializing...");
            // Important: Disable the legacy PIC so it doesn't interfere
            pic::disable();
            unsafe { init_apic() };
        } else {
            log_warn!("APIC not found. Falling back to legacy PIC...");
            pic::init()?;
        }
        Ok(())
    })?;
    Ok(())
}

pub fn send_eoi(irq: u8) {}
