pub struct TaskContext{
    pub sp: usize,
    pub ra: usize,
    s: [usize;12]
}

impl TaskContext {
    pub fn zero_cx(&self) -> TaskContext {
        Self{
            sp: 0,
            ra: 0,
            s: [0;12],
        }
    }

}