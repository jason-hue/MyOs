#![no_std]
#![feature(linkage)]

mod lang_items;
mod syscall;
pub mod console;
#[no_mangle]
#[link_section=".text.entry"]
pub extern "C" fn start_main(){
    clear_bss();
    exit(main());
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|a|{
        unsafe {
            (a as *mut u8).write_volatile(0);
        }
    })
}
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

use crate::syscall::{sys_exit, sys_write, sys_yield};

pub fn write(fd:usize, buffer:&[u8]) -> isize {
    sys_write(fd, buffer.as_ptr(),buffer.len())
}
pub fn exit(exit_code:i32)->isize{
    sys_exit(exit_code)
}
pub fn _yield() -> isize {
    sys_yield()
}