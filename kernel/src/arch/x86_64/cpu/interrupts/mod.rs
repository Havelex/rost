use crate::{
    cpu::interrupts::{
        GenericInterrupt, InterruptKind, exceptions::ExceptionType, handle_interrupt,
    },
    error::Result,
    init_step,
};

/// Vector number at which hardware (external) IRQs start.
/// Vectors 0–31 are reserved for CPU exceptions; vectors 32+ map to IRQ lines.
const HARDWARE_IRQ_VECTOR_BASE: u64 = 32;

pub mod apic;
mod idt;
pub mod pic;

/// CPU register state captured by the ISR stub before calling the Rust handler.
///
/// The assembly stub (`interrupts.S`) pushes all general-purpose registers plus
/// the vector number and error code onto the stack in this exact layout before
/// calling [`x86_64_interrupt_handler`].  The CPU itself pushes `rip`, `cs`,
/// `rflags`, `rsp`, and `ss` above them as part of the interrupt-gate protocol.
#[repr(C)]
pub struct InterruptContext {
    /// General-purpose register R15.
    pub r15: u64,
    /// General-purpose register R14.
    pub r14: u64,
    /// General-purpose register R13.
    pub r13: u64,
    /// General-purpose register R12.
    pub r12: u64,
    /// General-purpose register R11.
    pub r11: u64,
    /// General-purpose register R10.
    pub r10: u64,
    /// General-purpose register R9.
    pub r9: u64,
    /// General-purpose register R8.
    pub r8: u64,
    /// Base pointer register (RBP).
    pub rbp: u64,
    /// Destination index register (RDI).
    pub rdi: u64,
    /// Source index register (RSI).
    pub rsi: u64,
    /// Data register (RDX).
    pub rdx: u64,
    /// Counter register (RCX).
    pub rcx: u64,
    /// Base register (RBX).
    pub rbx: u64,
    /// Accumulator register (RAX).
    pub rax: u64,

    /// IDT vector number that fired (pushed by the ISR stub).
    pub vector: u64,
    /// CPU-provided error code for faults that supply one; zero otherwise.
    pub error_code: u64,

    // CPU-pushed fields (interrupt-gate protocol):
    /// Instruction pointer at the point of interruption.
    pub rip: u64,
    /// Code segment selector at the point of interruption.
    pub cs: u64,
    /// CPU flags register (RFLAGS) at the point of interruption.
    pub rflags: u64,
    /// Stack pointer at the point of interruption.
    pub rsp: u64,
    /// Stack segment selector at the point of interruption.
    pub ss: u64,
}

/// Common C-ABI entry point called by every ISR stub in `interrupts.S`.
///
/// Translates the raw vector number and error code in `ctx` into a
/// [`GenericInterrupt`] and forwards it to the portable [`handle_interrupt`]
/// dispatcher.  For CPU exceptions (vector < 32) a register dump is printed
/// before the dispatch (except for breakpoints, which are handled silently).
///
/// # Safety
/// Called exclusively from the ISR stubs in `interrupts.S`.  `ctx` must point
/// to a valid [`InterruptContext`] on the interrupt stack.
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
        InterruptKind::Hardware((ctx.vector - HARDWARE_IRQ_VECTOR_BASE) as u8)
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

/// Initialize the IDT and the legacy 8259 PIC.
///
/// Installs ISR stubs into the IDT, fires a test `int3` to verify the handler
/// works, remaps the PIC to vectors 0x20–0x2F, and masks all IRQ lines except
/// the cascade (IRQ 2).
///
/// # Returns
/// - `Ok(())` on success, or a [`KernelError`](crate::error::KernelError) on failure.
pub fn init() -> Result<()> {
    init_step("Initializing IDT", "IDT initialized", idt::init)?;
    init_step(
        "Testing breakpoint exception",
        "Successfully returned from breakpoint",
        || {
            unsafe {
                core::arch::asm!("int3");
            };
            Ok(())
        },
    )?;

    init_step("Initializing PIC", "PIC initialized", || {
        pic::init()?;
        // IRQ0 (PIT timer) is intentionally left masked here.  It will be
        // unmasked by pit::init() once the PIT counter has been fully
        // programmed, preventing spurious timer interrupts during early boot.
        pic::clear_mask(pic::IRQ_CASCADE); // IRQ2: cascade (required for slave PIC IRQs)
        Ok(())
    })?;
    Ok(())
}

/// Attempt to upgrade from the PIC to an APIC variant after paging is active.
///
/// Tries x2APIC → xAPIC in order, falling back silently to the PIC already
/// configured in `init()`.  Safe to call after `init_memory()`.
pub fn init_apic_post_paging() {
    apic::try_init_apic();
}

/// Forward an end-of-interrupt signal to the active interrupt controller.
///
/// # Parameters
/// - `irq`: The IRQ line number (0–15) that fired.
pub fn send_eoi(irq: u8) {
    apic::send_eoi(irq);
}
