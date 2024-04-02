use crate::syscall::fs::sys_write;
use crate::syscall::process::exit;

mod fs;
mod process;
const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
pub fn syscall(id: usize,arg0: usize,arg1: usize,arg2: usize)->isize{
    match id {
        SYS_WRITE=>{
            sys_write(arg0, arg1, arg2)
        }
        SYS_EXIT=>{
            exit(arg0 as i32)
        }
        _ => {
            panic!("Unsupported syscall_id: {}", id)
        }
    }
}