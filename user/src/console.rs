use crate::{sys_write, write};
use core::fmt::{Arguments, Write};
struct Stdout;
const STDOUT:usize = 1;

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