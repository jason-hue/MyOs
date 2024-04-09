use core::arch::global_asm;
use crate::task::context::TaskContext;
global_asm!(include_str!("switch.asm"));
extern "C"{
    pub fn _switch(cuttent_task: *mut TaskContext,next_task: *mut TaskContext);
}