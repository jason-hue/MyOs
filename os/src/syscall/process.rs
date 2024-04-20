use crate::task::{suspend_current_and_run_next, exit_current_and_run_next_task, change_program_brk};
use crate::timer::get_time_us;

pub fn exit(exit_code:i32)->!{
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next_task();
    panic!("Unreachable in sys_exit!");
}
pub fn sys_yield()->isize{
    suspend_current_and_run_next();
    0
}
pub fn sys_get_time() -> isize {
    get_time_us() as isize
}
pub fn sys_sbrk(size: i32) -> isize {
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}