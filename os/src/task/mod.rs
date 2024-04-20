use alloc::vec::Vec;
use lazy_static::*;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::sync::UPsafeCell;
use crate::loader::{get_app_data, get_app_num};
use crate::sbi::shutdown;
use crate::task::context::TaskContext;
use crate::task::switch::_switch;
use crate::trap::Context::TrapContext;

mod context;
mod switch;
mod task;

pub struct TaskManager{
    task_manager_inner: UPsafeCell<TaskManagerInner>,
    app_num: usize,
}
struct TaskManagerInner{
    current_task: usize,
    tasks: Vec<TaskControlBlock>
}
impl TaskManager {
    fn mark_current_suspended(&self){
        let mut  inner = self.task_manager_inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self){
        let mut  inner = self.task_manager_inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.task_manager_inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
    }

    fn run_next_task(&self){
        if let Some(next) = self.find_next_task(){
            let mut inner = self.task_manager_inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status= TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &mut inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                _switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        }else {
            println!("All applications completed!");
            shutdown(false);
        }

    }
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.task_manager_inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.app_num + 1).map(|id| id % self.app_num).find(|id|{
            inner.tasks[*id].task_status == TaskStatus::Ready
        })
    }
    fn run_first_task(&self)->!{
        let mut inner = self.task_manager_inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_cx();
        unsafe {
            _switch(
                &mut _unused as *mut TaskContext,
                next_task_cx_ptr
            );
        }
        panic!("unreachable in run_first_task!")
    }
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.task_manager_inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }
    fn get_current_token(&self) -> usize {
        let inner = self.task_manager_inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }


}
lazy_static!{
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_app_num();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(
                get_app_data(i),
                i,
            ));
        }
        TaskManager {
            app_num: num_app,
            task_manager_inner: unsafe {
                UPsafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };

}
pub fn suspend_current_and_run_next(){
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next_task(){
    mark_current_exited();
    run_next_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();

}
pub fn change_program_brk(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_brk(size)
}
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}