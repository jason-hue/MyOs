use crate::{get_char, write};
use core::fmt::{Arguments, Write};
struct Stdout;
const STDOUT: usize = 1;
const STDIN: usize = 0;

impl Write for Stdout{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(STDOUT,s.as_bytes());
        Ok(())
    }
}

pub fn print(args:Arguments){
    Stdout.write_fmt(args).unwrap()
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
#[macro_export]
macro_rules! logo {
    ($msg:expr) => {
        println!("\x1b[0;31m{}\x1B[0m",$msg);
    };
    ($($msg:expr),*)=>{
        println!("\x1b[0;31m{}\x1B[0m",format_args!($($msg),*));
    }
}
pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    get_char(STDIN, &mut c);
    c[0]
}