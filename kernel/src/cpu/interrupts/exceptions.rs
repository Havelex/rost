pub enum ExceptionType {
    DivideByZero,
    Breakpoint,
    GeneralProtectionFault(u64), // Contains the error code
    PageFault { addr: u64, error_code: u64 },
    Unknown(u64),
}

pub struct GenericExceptionInfo {
    pub rip: u64,
    pub exception: ExceptionType,
}

pub fn handle_generic_exception(info: GenericExceptionInfo) {
    match info.exception {
        ExceptionType::Breakpoint => {
            log_info!("Stopping at breakpoint: RIP={:#x}", info.rip);
        }
        ExceptionType::PageFault { addr, error_code } => {
            panic!(
                "PAGE FAULT at {:#x}\nAttempted to access: {:#x}\nError Flags: {:#b}",
                info.rip, addr, error_code
            );
        }
        ExceptionType::GeneralProtectionFault(code) => {
            panic!("GPF at {:#x} with error code {:#x}", info.rip, code);
        }
        ExceptionType::Unknown(vector) => {
            panic!(
                "Unkown CPU exception at {:#x} with vector {:#x}",
                info.rip, vector
            )
        }
        _ => panic!("Unhandled exception at {:#x}", info.rip),
    }
}
