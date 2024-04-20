use alloc::string::String;
use crate::{logo, task, timer, trap};
use crate::sbi::console_getchar;
pub const ENTER: u8 = 13;
pub const BACKSPACE: u8 = 127;
pub const MYOS_ASCII_ART: &str = r#"
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


pub fn shell() {
    logo!("{}",MYOS_ASCII_ART);
    println!("");
    print!("knifefire@knifefire-Legion-Y9000P-IAH7H:~ ");

    let mut buffer = String::new();
    loop {
        let mut line = read_line(&mut buffer);
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

fn process_command(command: &str) {
    let trimmed_command = command.trim();
    match trimmed_command {
        "help" | "?" | "h" => {
            println!("");
            println!("available commands:");
            println!("  run       run the built in app");
            println!("  help      print this help message  (alias: h, ?)");
            println!("  shutdown  shutdown the machine     (alias: sd, exit)");
        }
        "shutdown" | "sd" | "exit" => crate::sbi::shutdown(false),
        "run" => {
            task::run_first_task();
        }
        "" => {}
        _ => {
            println!("");
            println!("unknown command: {}",command);
        }
    };
}
// Manage the cursor manually
fn read_line(buffer: &mut String) -> u8 {
    let mut cursor_position = buffer.len();

    loop {
        let ch = console_getchar() as u8;
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