#![no_std]
#![feature(linkage)]
#[no_mangle]
#[link_section=".text.entry"]
pub extern "C" fn start_main(){
    clear_bss();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a|{
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