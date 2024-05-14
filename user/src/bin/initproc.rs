#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{exec, fork, wait, _yield};

#[no_mangle]
fn main() -> i32 {
    let file: [&str; 5] =["fantastic_text\0","hello_world\0","matrix\0","sleep\0","exit\0"];

    for f in &file{
        fe(f);
    }

    0
}

fn fe(file: &str){
    if fork()==0{
        exec(file);
    }else {
        let mut exit_code: i32 = 0;
        let pid = wait(&mut exit_code);
        return;
    }
}