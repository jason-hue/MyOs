use core::arch::{asm, global_asm};
use crate::trap::Context::TrapContext;
use core::slice::{from_raw_parts,from_raw_parts_mut};
use log::Level::Error;
use crate::{error, info};

global_asm!(include_str!("link_app.asm"));

const MAX_APP: usize =16 ;
const BASE_ADDRESS:usize = 0x80400000;
const USER_STACK_SIZE: usize = 4096*2;
const KERNEL_STACK_SIZE: usize = 4096*2;
const APP_MAX_SIZE: usize = 0x20000;
pub fn get_app_num() -> usize {
    extern "C"{
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let app_num = unsafe { num_app_ptr.read_volatile() };
    app_num
}
pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" { fn _num_app(); }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_app_num();
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };
    println!("app {} load",app_id);
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id]
        )
    }

}