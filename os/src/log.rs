#[macro_export]
macro_rules! info {
    ($msg:expr) => {
        println!("\x1B[34mINFO:{}\x1B[0m",$msg);
    };
    ($($msg:expr),*)=>{
        println!("\x1B[34mINFO:{}\x1B[0m",format_args!($($msg),*));
    }
}
#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        println!("\x1B[31mERROR:{}\x1B[0m",$msg);
    };
    ($($msg:expr),*)=>{
        println!("\x1B[31mERROR:{}\x1B[0m",format_args!($($msg),*));
    }
}
#[macro_export]
macro_rules! warn {
    ($msg:expr) => {
        println!("\x1B[33mWARNING:{}\x1B[0m",$msg);
    };
    ($($msg:expr),*)=>{
        println!("\x1B[33mWARNING:{}\x1B[0m",format_args!($($msg),*));
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

#[macro_export]
macro_rules! applogo {
    ($msg:expr) => {
        println!("\x1B[32m{}\x1B[0m",$msg);
    };
    ($($msg:expr),*)=>{
        println!("\x1b[0;32m{}\x1B[0m",format_args!($($msg),*));
    }
}