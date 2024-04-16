pub const MAX_APP: usize =16 ;
pub const BASE_ADDRESS:usize = 0x80400000;
pub const USER_STACK_SIZE: usize = 4096*2;
pub const KERNEL_STACK_SIZE: usize = 4096*2;
pub const APP_MAX_SIZE: usize = 0x20000;
pub const qemu_CLOCK_FRED: usize = 12500000;
pub const k210_CLOCK_FRED: usize = 403000000 / 62;