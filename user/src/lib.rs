#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

mod lang_items;
mod syscall;
pub mod console;
use buddy_system_allocator::LockedHeap;
use syscall::*;

const USER_HEAP_SIZE: usize = 16384;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}


#[no_mangle]
#[link_section=".text.entry"]
pub extern "C" fn start_main(){
    clear_bss();
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
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

use crate::syscall::{sys_exec, sys_exit, sys_fork, sys_get_time, sys_getpid, sys_read, sys_sbrk, sys_waitpid, sys_write, sys_yield};

pub fn write(fd:usize, buffer:&[u8]) -> isize {
    sys_write(fd, buffer.as_ptr(),buffer.len())
}
pub fn exit(exit_code:i32)->isize{
    sys_exit(exit_code)
}
pub fn _yield() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}

pub fn sbrk(size: i32) -> isize {
    sys_sbrk(size)
}
pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)

}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                _yield();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                _yield();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}
pub fn shutdown(){
    sys_shutdown();
}
pub fn print_apps(){
    sys_print_apps();
}