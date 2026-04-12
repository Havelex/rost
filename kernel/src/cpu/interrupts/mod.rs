use spin::Mutex;

pub static INTERRUPTS: Mutex<InterruptManager> = Mutex::new(InterruptManager::new());

#[derive(Debug, Clone, Copy)]
pub struct InterruptFrame {
    pub ip: usize,
    pub sp: usize,
    pub vector: u8,
    pub error_code: Option<u64>,
}

pub type InterruptHandler = fn(InterruptFrame);

pub struct InterruptManager {
    handlers: [Option<InterruptHandler>; 256],
}

impl InterruptManager {
    pub const fn new() -> Self {
        Self {
            handlers: [None; 256],
        }
    }

    pub fn register(&mut self, vector: u8, handler: InterruptHandler) {
        self.handlers[vector as usize] = Some(handler);
    }

    pub fn dispatch(&self, frame: InterruptFrame) {
        if let Some(handler) = self.handlers[frame.vector as usize] {
            handler(frame);
        } else {
            crate::println!("Unhandled interrupt: {}", frame.vector);
        }
    }
}
