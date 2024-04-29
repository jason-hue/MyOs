use core::arch::asm;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
const SYS_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYS_SBRK: usize = 214;
const SYS_GETPID: usize = 172;
const SYS_FORK: usize = 220;
const SYS_EXEC: usize = 221;
const SYS_WAITPID: usize = 260;
const SYT_SHUTDOWN: usize = 48;
const SYS_PRINT_APPS: usize = 100;
const SYS_OPEN: usize = 56;
const SYS_CLOSE: usize = 57;
const SYS_READ: usize = 63;
const SYS_GETCHAR: usize = 520;


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


pub fn sys_exit(exit_code: i32) ->isize{
    syscall(SYS_EXIT,[exit_code as usize,0,0])
}

pub fn sys_yield()->isize{
    syscall(SYS_YIELD,[0,0,0])
}
pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}
pub fn sys_sbrk(size: i32) -> isize {
    syscall(SYS_SBRK, [size as usize, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYS_GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYS_FORK, [0, 0, 0])
}

pub fn sys_exec(path: *const u8) -> isize {
    syscall(SYS_EXEC, [path as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYS_WAITPID, [pid as usize, exit_code as usize, 0])
}
pub fn sys_shutdown(){
    syscall(SYT_SHUTDOWN,[0,0,0]);
}
pub fn sys_print_apps(){
    syscall(SYS_PRINT_APPS,[0,0,0]);
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYS_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYS_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}
pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYS_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYS_CLOSE, [fd, 0, 0])
}

pub fn sys_getchar(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYS_GETCHAR,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}