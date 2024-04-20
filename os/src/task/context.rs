use crate::trap::trap_return;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
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
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }

}