use crate::batch::run_next_app;

pub fn exit(exit_code:i32) -> isize {
    println!("[kernel] Application exited with code {}", exit_code);
    run_next_app();
    1
}