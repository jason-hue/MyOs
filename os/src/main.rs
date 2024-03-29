#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
mod console;
mod sbi;
mod log;
mod k210_lcd_driver;



use core::panic::PanicInfo;
use core::arch::global_asm;
use crate::console::print;
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
    info!("hello_world!");
    error!("hello_world！");
    warn!("hello_world!");
    panic!("shutdown machine!");
    loop {

    }
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