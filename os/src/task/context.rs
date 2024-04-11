#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TaskContext{
    pub ra: usize,
    pub sp: usize,
    s: [usize;12]
}

impl TaskContext {
    pub fn zero_cx() -> Self {
        Self{
            ra: 0,
            sp: 0,
            s: [0;12],
        }
    }
    pub fn go_to_restore(kernel_stack_ptr: usize) -> Self {
        extern "C"{
            fn _restore();
        }
        Self{
            ra: _restore as usize,
            sp: kernel_stack_ptr,
            s: [0;12],
        }
    }

}