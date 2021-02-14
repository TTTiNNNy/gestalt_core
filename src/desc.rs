
#[allow(dead_code)]
pub use crate::arch::arm_m4::context;
pub use crate::arch::arm_m4::context::{save_sw_context,change_context};
pub use crate::context::{ContextStatus};

pub const FREQ_CORE:        u32     = 64_000_000;
pub const TASK_NUM:         usize   = 10;
pub const INTERRUPT_NUM:    usize   = 14;
pub const THREAD_NUM:       usize   = 4;
pub const CPU_NUM:          usize   = 1;
