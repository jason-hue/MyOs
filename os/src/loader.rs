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
#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8;USER_STACK_SIZE]
}
#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack{
    data: [u8;KERNEL_STACK_SIZE]
}
impl UserStack{
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}
impl KernelStack{
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> usize {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe {
            cx_ptr as usize
        }
    }
}
static KERNEL_STACK: [KernelStack;MAX_APP] = [KernelStack{
    data: [0;KERNEL_STACK_SIZE]
};MAX_APP];
static USER_STACK: [UserStack;MAX_APP] = [UserStack{
    data: [0;USER_STACK_SIZE]
};MAX_APP];
pub fn get_app_num() -> usize {
    extern "C"{
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let app_num = unsafe { num_app_ptr.read_volatile() };
    app_num
}
fn get_base_address(app_id:usize) -> usize {
    BASE_ADDRESS+app_id*APP_MAX_SIZE
}
pub unsafe fn load_apps(){
    extern "C"{
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let app_num = get_app_num();
    let app_dst = from_raw_parts(num_app_ptr.add(1), app_num+1);
    info!("app sourece addr: {:#x?}",app_dst);
    for i in 0..app_num{
        let address = get_base_address(i);
        (address..address + APP_MAX_SIZE).for_each(|addr|{
            unsafe {
                (addr as *mut u8).write_volatile(0);
            }
        });
        let src = from_raw_parts(app_dst[i] as *const u8,app_dst[i+1]-app_dst[i]);
        let dst = from_raw_parts_mut(address as *mut u8,src.len());
        info!("app{}addr={:02X}",i,address);
        dst.copy_from_slice(src);

    }
    unsafe {
        asm!("fence.i");
    }
}
pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(TrapContext::init_trap_context(
        get_base_address(app_id),
        USER_STACK[app_id].get_sp(),
    ))
}