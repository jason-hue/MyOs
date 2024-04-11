use core::arch::global_asm;
use riscv::register::{scause, stval};
use riscv::register::scause::Trap;
use riscv::register::scause::Exception;
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec;
use crate::trap::Context::TrapContext;
use crate::syscall::syscall;

pub mod Context;
global_asm!(include_str!("trap.asm"));

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) ->&mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall)=> {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17],cx.x[10],cx.x[11],cx.x[12]) as usize;
        }
        Trap::Exception(Exception::StorePageFault) | Trap::Exception(Exception::StoreFault)=>{
            println!("[kernel] PageFault in application, kernel killed it.");
            panic!("[kernel] Cannot continue!");
        }
        Trap::Exception(Exception::IllegalInstruction)=>{
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            panic!("[kernel] Cannot continue!");
        }
        _=>{
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}

pub fn init(){
    extern "C"{
        fn _alltraps();
    }
    unsafe {
        stvec::write(_alltraps as usize,TrapMode::Direct);
    }
}