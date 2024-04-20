pub const MAX_APP: usize =16 ;
pub const BASE_ADDRESS:usize = 0x80400000;
pub const USER_STACK_SIZE: usize = 4096*2;
pub const KERNEL_STACK_SIZE: usize = 4096*2;
pub const APP_MAX_SIZE: usize = 0x20000;
pub const qemu_CLOCK_FRED: usize = 12500000;
pub const k210_CLOCK_FRED: usize = 403000000 / 62;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;//一个页占12位字节地址
pub const MEMORY_END: usize = 0x80800000;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}