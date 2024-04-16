use alloc::string::String;

pub const ENTER: u8 = 13;
pub const BACKSPACE: u8 = 127;

pub fn shell() {
    print!("shell> ");

    let mut command = String::new();

    loop {
        match crate::sbi::console_getchar() {
            ENTER => {
                println!(" ");
                process_command(&command);
                command.clear();
                print!("shell> ");
            }
            BACKSPACE => {
                if command.len() > 0 {
                    command.pop();
                    print!("{}", BACKSPACE as char)
                }
            }
            _=>{}
        }
    }
}

fn process_command(command: &str) {
    match command {
        "help" | "?" | "h" => {
            println!("available commands:");
            println!("  help      print this help message  (alias: h, ?)");
            println!("  shutdown  shutdown the machine     (alias: sd, exit)");
        }
        "shutdown" | "sd" | "exit" => crate::sbi::shutdown(false),
        "" => {}
        _ => {
            println!("unknown command: {}",command);
        }
    };
}
