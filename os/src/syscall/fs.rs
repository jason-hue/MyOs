use alloc::string::ToString;
use alloc::sync::Arc;
use crate::fs::{File, FileDescriptor, open_file, OpenFlags};
use crate::memory::page_table::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token, suspend_current_and_run_next};
const STDOUT: usize = 1;
// pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
//     match fd {
//         FD_STDOUT => {
//             let buffers = translated_byte_buffer(current_user_token(), buf, len);
//             for buffer in buffers {
//                 print!("{}", core::str::from_utf8(buffer).unwrap());
//             }
//             len as isize
//         },
//         _ => {
//             panic!("Unsupported fd in sys_write!");
//         }
//     }
// }
use crate::sbi::console_getchar;

const FD_STDIN: usize = 0;

// pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
//     match fd {
//         FD_STDIN => {
//             assert_eq!(len, 1, "Only support len = 1 in sys_read!");
//             let mut c: usize;
//             loop {
//                 c = console_getchar();
//                 if c == 0 {
//                     suspend_current_and_run_next();
//                     continue;
//                 } else {
//                     break;
//                 }
//             }
//             let ch = c as u8;
//             let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
//             unsafe { buffers[0].as_mut_ptr().write_volatile(ch); }
//             1
//         }
//         _ => {
//             panic!("Unsupported fd in sys_read!");
//         }
//     }
// }
pub fn sys_close(fd: usize) -> isize {
    println!("enter close");
    let process = current_task().unwrap();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}
pub fn sys_open(fd:isize, path: *const u8, flags: u32) -> isize {
    print!("");
    let process = current_task().unwrap();
    let token = current_user_token();
    let mut inner = process.inner_exclusive_access();
    let path = translated_str(token, path).replace("./", "");
    let flag = OpenFlags::from_bits(flags).unwrap();
    let (readable, writable) = flag.read_write();
    let dir = if fd >= 0 {
        let fd = fd as usize;
        if let Some(dir) = inner.fd_table.get(fd).unwrap() {
            dir
        } else {
            return -1;
        }
    } else {
        &inner.work_dir.as_ref()
    };
    //打开当前
    if path == ".".to_string() {
        let tmp = dir.clone();
        drop(dir);
        let fd = inner.alloc_fd();
        inner.fd_table.insert(fd, Some(tmp));
        return fd as isize;
    }
    let flag = OpenFlags::from_bits(flags).unwrap();
    let file = if flag.contains(OpenFlags::CREATE) {
        dir.create(&path, readable, writable, false)
    } else {
        open_file(&path, flag)
        //dir.open(&path, readable, writable, directory)
    };
    if let Some(file) = file {
        let fd = inner.alloc_fd();
        inner.fd_table.insert(fd, Some(FileDescriptor::File(file)));
        fd as isize
    } else {
        -1
    }
}
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_task().unwrap();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    if fd != 1 {println!("write fd {}",fd);}
    let token = current_user_token();
    let process = current_task().unwrap();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}