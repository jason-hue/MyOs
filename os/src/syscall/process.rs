use crate::task::{suspend_current_and_run_next,exit_current_and_run_next_task};

pub fn exit(exit_code:i32)->!{
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next_task();
    panic!("Unreachable in sys_exit!");
}
pub fn sys_yield()->isize{
    suspend_current_and_run_next();
    0
}