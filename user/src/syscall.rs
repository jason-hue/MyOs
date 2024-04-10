use core::arch::asm;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
const SYS_YIELD: usize = 124;
pub fn syscall(sys_id:usize, arg:[usize;3])->isize{
    let mut ret;
    unsafe { asm!(
        "ecall",
        inlateout("a0") arg[0] => ret,
        in("a1") arg[1],
        in("a2") arg[2],
        in("a7") sys_id,
    ) }
    ret
}

pub fn sys_write(fd:usize, buffer: *const u8, len:usize) -> isize {
    syscall(SYS_WRITE,[fd,buffer as usize,len])
}
pub fn sys_exit(exit_code: i32) ->isize{
    syscall(SYS_EXIT,[exit_code as usize,0,0])
}

pub fn sys_yield()->isize{
    syscall(SYS_YIELD,[0,0,0])
}