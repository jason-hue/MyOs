use crate::syscall::fs::{sys_read, sys_write};
use crate::syscall::process::{exit, sys_exec, sys_fork, sys_get_time, sys_getpid, sys_shutdown, sys_waitpid, sys_yield};

mod fs;
mod process;
const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
const SYS_YIELD: usize = 124;
const SYS_GET_TIME: usize = 169;
const SYS_SBRK: usize = 214;
const SYS_GETPID: usize = 172;
const SYS_FORK: usize = 220;
const SYS_EXEC: usize = 221;
const SYS_WAITPID: usize = 260;
const SYS_READ: usize = 63;
const SYS_SHUTDOWN: usize = 48;
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYS_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYS_EXIT => exit(args[0] as i32),
        SYS_YIELD => sys_yield(),
        SYS_GET_TIME => sys_get_time(),
        SYS_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYS_GETPID => sys_getpid(),
        SYS_FORK => sys_fork(),
        SYS_EXEC => sys_exec(args[0] as *const u8),
        SYS_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYS_SHUTDOWN => sys_shutdown(false),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}