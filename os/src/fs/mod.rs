#![allow(unused)]
mod inode;
mod pipe;
mod stdio;
mod file_descriptor;

use crate::{fatfs::io::SeekFrom, memory::page_table::UserBuffer};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn seek(&self, _offset: SeekFrom) -> usize {
        0
    }
    fn read(&self, buf: UserBuffer) -> usize {
        0
    }
    fn write(&self, buf: UserBuffer) -> usize {
        0
    }
    fn create(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        None
    }
    fn open(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        None
    }
    fn remove(&self, path: &str) -> bool {
        false
    }
    fn islink(&self) -> bool {
        false
    }
    fn kstat(&self, stat: &mut Kstat) {}
    fn name(&self) -> String {
        "/".to_string()
    }
    fn getdents(&self, dirent: &mut Dirent) -> isize {
        -1
    }
}

use alloc::{string::{String, ToString}, sync::Arc};
pub use inode::{open_file, root, Dirent, Kstat, OSInode, OpenFlags};
pub use pipe::{make_pipe, Pipe};
pub use stdio::{Stdin, Stdout};
pub use file_descriptor::FileDescriptor;

// 等待实现的VFS
pub trait VFS {
    fn open();
    fn create();
    fn remove();
    fn mount();
    fn umount();
    fn fstat();
}

