use core::arch::global_asm;
use lazy_static::lazy_static;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::sync::UPsafeCell;
use crate::config::MAX_APP;
use crate::loader::{get_app_num, init_app_cx};
use crate::sbi::shutdown;
use crate::task::context::TaskContext;
use crate::task::switch::_switch;
mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

pub struct TaskManager{
    task_manager_inner: UPsafeCell<TaskManagerInner>,
    app_num: usize,
}
struct TaskManagerInner{
    current_task: usize,
    tasks: [TaskControlBlock;MAX_APP]
}
lazy_static!{
    static ref TASK_MANAGER: TaskManager = {

        let num_app = get_app_num();
        let mut tasks = [TaskControlBlock{
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_cx(),
        };MAX_APP];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::go_to_restore(init_app_cx(i));
            task.task_status = TaskStatus::Ready;
        }
        TaskManager{
            app_num: num_app,
            task_manager_inner: UPsafeCell::new(
                TaskManagerInner{
                    current_task: 0,
                    tasks,
                }
            )
        }

    };

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
    fn run_first_task(&self){
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


}
pub fn suspend_current_and_run_next(){
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next_task(){
    mark_current_suspended();
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
