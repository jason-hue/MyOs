use crate::trap::Context::TrapContext;

#[derive(Clone, Copy, Debug)]
pub struct TaskContext{
    pub sp: usize,
    pub ra: usize,
    s: [usize;12]
}

impl TaskContext {
    pub fn zero_cx() -> Self {
        Self{
            sp: 0,
            ra: 0,
            s: [0;12],
        }
    }
    pub fn go_to_restore(kernel_stack_ptr: usize) -> Self {
        extern "C"{
            fn _restore();
        }
        Self{
            sp: kernel_stack_ptr,
            ra: _restore as usize,
            s: [0;12],
        }
    }

}