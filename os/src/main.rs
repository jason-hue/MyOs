#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(str_from_raw_parts)]

#[macro_use]
mod console;
mod sbi;
mod log;
mod k210_lcd_driver;
mod loader;
mod sync;
mod trap;
mod syscall;
mod task;
mod config;



use core::panic::PanicInfo;
use core::arch::global_asm;
use ::log::{debug, trace};
use log::*;
use crate::console::print;
use crate::k210_lcd_driver::ST7789VConfig;
use crate::sbi::shutdown;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location)=_info.location(){
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            _info.message().unwrap()
        )
    }else {
        println!("Panicked: {}", _info.message().unwrap())
    }
    shutdown(true);
}
global_asm!(include_str!("entry.asm"));
#[no_mangle]
pub extern "C" fn  start_main(){
    clear_bss();
    extern "C"{
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn stack_low();
        fn stack_top();
        fn stack_bss();

    }
    info!("hello_world!");
    error!("hello_world！");
    warn!("hello_world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top_bottom={:#x}, lower_bound={:#x}",
        stack_top as usize, stack_low as usize
    );
    info!("[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize);
    info!("[kernel] .rodata [{:#x}, {:#x})",srodata as usize,erodata as usize);
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    info!("[kernel] .stack_bss {:#x}",stack_bss as usize);
    trap::init();
    unsafe { loader::load_apps(); }
    task::run_first_task();

    panic!("shutdown machine!");
}
fn clear_bss(){
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a|{
        unsafe{(a as *mut u8).write_volatile(0)}
    })
}
