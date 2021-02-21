

pub use crate::arch::arm_m4::context;
pub use crate::arch::arm_m4::context::{save_sw_context,change_context};
pub use crate::context::{ContextStatus};
#[allow(dead_code)]
pub const FREQ_CORE:        u32     = 64_000_000;
#[allow(dead_code)]
pub const TASK_NUM:         usize   = 10;
#[allow(dead_code)]
pub const INTERRUPT_NUM:    usize   = 14;
#[allow(dead_code)]
pub const THREAD_NUM:       usize   = 4;
#[allow(dead_code)]
pub const CPU_NUM:          usize   = 1;
