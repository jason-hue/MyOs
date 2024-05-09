#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate user_lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

use alloc::string::String;
use user_lib::console::getchar;
use user_lib::{exec, fork, print_apps, shutdown, waitpid};
const ENTER: u8 = 13;
const BACKSPACE: u8 = 127;
const MYOS_ASCII_ART: &str = r#"
MMMMMMMM               MMMMMMMMYYYYYYY       YYYYYYY        OOOOOOOOO     SSSSSSSSSSSSSSS
M:::::::M             M:::::::MY:::::Y       Y:::::Y      OO:::::::::OO SSS:::::::::::::S
M::::::::M           M::::::::MY:::::Y       Y:::::Y    OO:::::::::::::OSS:::::::::::::::S
M:::::::::M         M:::::::::M Y::::::Y     Y::::::Y   O:::::::OOO:::::::OS:::::SSSSSS::::::S
M::::::::::M       M::::::::::M  YYY:::::Y   Y:::::YYY  O::::::O   O::::::OS:::::S     SSSSSSS
M:::::::::::M     M:::::::::::M    Y:::::Y Y:::::Y    O:::::O     O:::::OS:::::S
M:::::::M::::M   M::::M:::::::M     Y:::::Y:::::Y     O:::::O     O:::::O S:::::S
M::::::M M::::M M::::M M::::::M      Y:::::::::Y      O:::::O     O:::::O  S:::::S
M::::::M  M::::M::::M  M::::::M       Y:::::::Y       O:::::O     O:::::O   S::::SSSS
M::::::M   M:::::::M   M::::::M        Y:::::Y        O:::::O     O:::::O    SS::::::SSSSS
M::::::M    M:::::M    M::::::M        Y:::::Y        O:::::O     O:::::O      SSS::::::::SS
M::::::M     MMMMM     M::::::M        Y:::::Y        O::::::O   O::::::O         SSSSSS::::S
M::::::M               M::::::M        Y:::::Y        O:::::::OOO:::::::O              S:::::S
M::::::M               M::::::M        Y:::::Y         OO:::::::::::::OO              S:::::S
M::::::M               M::::::M      YYYY:::::YYYY        OO:::::::::OO               S:::::S
MMMMMMMM               MMMMMMMM      YYYYYYYYYYYYY          OOOOOOOOO                 SSSSSSS
"#;

#[no_mangle]
pub fn main()->! {
    logo!("{}",MYOS_ASCII_ART);
    println!("");
    print!("knifefire@knifefire-Legion-Y9000P-IAH7H:~ ");

    let mut buffer = String::new();
    loop {
        let mut line = read_line(&mut buffer);
        println!("");
        match line{
            ENTER => {
                process_command(buffer.as_ref());
                println!("");
                buffer.clear();
                print!("knifefire@knifefire-Legion-Y9000P-IAH7H:~ ");
            }
            _ => {}
        }
    }
}

fn process_command(command: &str){
    let trimmed_command = command.trim();
    match trimmed_command {
        "help" | "?" | "h" => {
            println!("");
            println!("available commands:");
            println!("  list      dispaly apps");
            println!("  run       run the built in app");
            println!("  help      print this help message  (alias: h, ?)");
            println!("  shutdown  shutdown the machine     (alias: sd, exit)");
        }
        "list" =>{
            print_apps();
        }
        "shutdown" | "sd" => shutdown(),
        "" => {}
        _ => {
            if !command.is_empty() {
                let pid = fork();
                if pid == 0{
                    if exec(trimmed_command) == -1{
                        println!("Error when executing!");
                        return;
                    }
                    unreachable!();
                }else {
                    let mut exit_code: i32 = 0;
                    let exit_pid = waitpid(pid as usize, &mut exit_code);
                    assert_eq!(pid, exit_pid);
                    println!("Shell: Process {} exited with code {}", pid, exit_code);

                }
            }
        }
    }
}
// Manage the cursor manually
fn read_line(buffer: &mut String) -> u8 {
    let mut cursor_position = buffer.len();

    loop {
        let ch = getchar() as u8;
        if ch == ENTER {
            return ENTER;
        } else if ch == BACKSPACE {
            if buffer.len() > 0 && cursor_position > 0 {
                buffer.remove(cursor_position - 1);
                cursor_position -= 1;
                print!("\x08 \x08");
            }
        } else {
            buffer.insert(cursor_position, ch as char);
            cursor_position += 1;
            print!("{}", ch as char);
        }
    }
}