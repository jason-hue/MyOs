mod context;
mod manager;
mod pid;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::loader::get_app_data_by_name;
use crate::sbi::shutdown;
use alloc::sync::Arc;
use lazy_static::*;
pub use manager::{fetch_task, TaskManager};
use switch::_switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;
pub use manager::add_task;
pub use pid::{pid_alloc, KernelStack, PidAllocator, Pid};
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    Processor,
};
pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
    schedule(task_cx_ptr);
}

pub const IDLE_PID: usize = 0;

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();
    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            shutdown(true)
        } else {
            shutdown(false)
        }
    }

    let mut inner = task.inner_exclusive_access();
    inner.task_status = TaskStatus::Zombie;
    inner.exit_code = exit_code;
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    inner.children.clear();
    inner.memory_set.recycle_data_pages();
    drop(inner);
    drop(task);
    let mut _unused = TaskContext::zero_cx();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = {
        let v = include_bytes!("../initproc");
        // let v = unsafe{inode.read_all()};
        // let vs = v.as_slice();
        Arc::new(TaskControlBlock::new(v.as_slice()))
    };
}
pub fn add_initproc() {
    add_task(INITPROC.clone());
}