use crate::task::context::TaskContext;

struct TaskControlBlock{
    task_status: TaskStatus,
    task_cx: TaskContext,
}
enum TaskStatus{
    UnInit,
    Ready,
    Running,
    Exited,
}