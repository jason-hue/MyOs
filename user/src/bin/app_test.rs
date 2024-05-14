#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{exec, exit, fork, wait, waitpid, sleep};
#[no_mangle]
fn main() -> i32 {
    let files: [&str; 5] =["fantastic_text\0","hello_world\0","matrix\0","sleep\0","exit\0"];
    for file in &files{
        fe(file);
    }
    0
}

fn fe(file: &str){
    let pid = fork();
    if pid == 0{
        if exec(file) == -1{
            println!("Error when executing!");
            return;
        }
        unreachable!();
    }else {
        let mut exit_code: i32 = 0;
        let exit_pid = waitpid(pid as usize, &mut exit_code);
        assert_eq!(pid, exit_pid);
        println!("Shell: Process {} exited with code {}", pid, exit_code);
        return;
    }
}