#![allow(unused)]
const SBI_CONSOLE_PUTCHAR:usize = 1;
use core::arch::asm;

#[inline(always)]
fn sbi_call(which: usize,arg0: usize,arg1: usize,arg2: usize)->usize{
        let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        )
    }
    ret
}
pub fn console_putchar(c: usize){
    sbi_call(SBI_CONSOLE_PUTCHAR,c,0,0);
}
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}