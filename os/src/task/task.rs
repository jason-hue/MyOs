use crate::task::context::TaskContext;

#[derive(Debug,Copy, Clone)]
pub struct TaskControlBlock{
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
}
#[derive(Clone, Copy, Debug,PartialEq)]
pub enum TaskStatus{
    UnInit,
    Ready,
    Running,
    Exited,
}