use core::slice::from_raw_parts;
use core::str::from_utf8;

const STDOUT: usize = 1;
pub fn sys_write(fd: usize, buffer: usize, len: usize) -> isize {
    unsafe {
        match fd {
            STDOUT => {
                let slice = from_raw_parts(buffer as *const u8, len);
                let str = from_utf8(slice).unwrap();
                print!("{}",str);
                len as isize

            }
            _ => {
                println!("Unsupport SystemWrite!!");
                0
            }
        }
    }

}