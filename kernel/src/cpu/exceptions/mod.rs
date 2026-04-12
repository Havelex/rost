pub enum Exception {
    DivideByZero,
    InvalidOpcode,
    GeneralProtectionFault,
    PageFault(PageFaultInfo),
    Breakpoint,
    Overflow,
    DoubleFault,
    NonMaskableInterrupt,
    StackFault,
    AlignmentFault,
    Unknown(u64),
}

pub trait ExceptionHandler {
    fn handle(&mut self, exception: Exception, ctx: &mut ExceptionContext);
}

pub struct ExceptionContext {
    pub ip: u64,
    pub sp: u64,
    pub flags: u64,
    pub error_code: Option<u64>,
    pub privilege_level: u64,
    // pub raw_vector: u64,
}

pub fn dispatch_exception(
    handler: &mut dyn ExceptionHandler,
    exception: Exception,
    ctx: &mut ExceptionContext,
) {
    handler.handle(exception, ctx);
}

// ========
// REFACTOR:
// ========
pub struct PageFaultInfo {
    pub address: u64,
    pub is_present: bool,
    pub is_write: bool,
    pub is_user: bool,
    pub is_exec: bool,
}
