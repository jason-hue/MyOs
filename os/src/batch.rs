use core::arch::{asm, global_asm};
use core::ops::Add;
use lazy_static::lazy_static;
use core::slice::{from_raw_parts,from_raw_parts_mut};
use crate::sbi::shutdown;
global_asm!(include_str!("link_app.S"));
use crate::sync::UPsafeCell;
use crate::trap::Context::TrapContext;

const MAX_APP: usize =16 ;
const BASE_ADDRESS:usize = 0x80400000;
const USER_STACK_SIZE: usize = 4096*2;
const KERNEL_STACK_SIZE: usize = 4096*2;
struct UserStack {
    data: [usize;USER_STACK_SIZE]
}
struct KernelStack{
    data: [usize;KERNEL_STACK_SIZE]
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
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe {
            cx_ptr.as_mut().unwrap()
        }
    }
}
static KERNEL_STACK: KernelStack = KernelStack{
    data: [0;KERNEL_STACK_SIZE]
};
static USER_STACK: UserStack = UserStack{
    data: [0;USER_STACK_SIZE]
};
struct AppManager {
    num_app: usize,
    current_app:usize,
    app_start:[usize;MAX_APP+1],
}
lazy_static!{
    static ref APP_MANAGER: UPsafeCell<AppManager> = unsafe{
        UPsafeCell::new({
            extern "C"{
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start:[usize;MAX_APP+1] = [0;MAX_APP+1];
            let app_raw = from_raw_parts(num_app_ptr.add(1),num_app+1);
            app_start[..=num_app].copy_from_slice(app_raw);
            AppManager{
                num_app,
                current_app:0,
                app_start,
            }
        })
    };
}
impl AppManager{
    unsafe fn load_app(&self,app_id:usize){
        if app_id>self.num_app{
            println!("All Applications completed!");
            shutdown(false);
        }
        println!("[kernel] Loading app_{}", app_id);
        from_raw_parts_mut(BASE_ADDRESS as *mut u8,MAX_APP+1).fill(0);
        let app_src = from_raw_parts(self.app_start[app_id] as *const u8,self.app_start[app_id+1]-self.app_start[app_id]);
        let mut app_dst = from_raw_parts_mut(BASE_ADDRESS as *mut u8,self.app_start[app_id+1]-self.app_start[app_id]);
        app_dst.copy_from_slice(app_src);
        asm!("fence.i");
    }
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }
    fn get_current_app(&self)->usize{
        self.current_app
    }
    fn move_to_next_app(&mut self){
        self.current_app += 1
    }
}
pub fn run_next_app()->!{
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(current_app)
    }
    app_manager.move_to_next_app();
    drop(app_manager);
    extern "C"{
        fn _restore(trap_context_addr: usize);
    }
    unsafe {
        _restore(KERNEL_STACK.push_context(
            TrapContext::init_trap_context(BASE_ADDRESS, USER_STACK.get_sp())
        ) as *const _ as usize);

    };
    panic!("Unreachable in batch::run_current_app!");

}
pub fn init(){
    print_app_info();
}
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}
