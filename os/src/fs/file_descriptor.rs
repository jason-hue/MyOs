use alloc::sync::Arc;

use super::{File, OSInode};
use crate::mm::UserBuffer;

#[derive(Clone)]
// 抽象的文件描述符
// OSInode是具体的文件
// Abstract 主要是提供给 fd_table 以及 管道所使用
// 在这里FileDescriptor起分发的作用
pub enum FileDescriptor {
    File(Arc<OSInode>),
    Abstract(Arc<dyn File + Send + Sync>),
}

impl File for FileDescriptor {
    fn readable(&self) -> bool {
        match self {
            FileDescriptor::File(inode) => inode.readable(),
            FileDescriptor::Abstract(inode) => inode.readable(),
        }
    }
    fn writable(&self) -> bool {
        match self {
            FileDescriptor::File(inode) => inode.writable(),
            FileDescriptor::Abstract(inode) => inode.writable(),
        }
    }
    fn read(&self, buf: UserBuffer) -> usize {
        match self {
            FileDescriptor::File(inode) => inode.read(buf),
            FileDescriptor::Abstract(inode) => inode.read(buf),
        }
    }
    fn write(&self, buf: UserBuffer) -> usize {
        match self {
            FileDescriptor::File(inode) => inode.write(buf),
            FileDescriptor::Abstract(inode) => inode.write(buf),
        }
    }
    fn open(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        match self {
            FileDescriptor::File(inode) => inode.open(name, read, write, isdir),
            FileDescriptor::Abstract(inode) => inode.open(name, read, write, isdir),
        }
    }
    fn create(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        match self {
            FileDescriptor::File(inode) => inode.create(name, read, write, isdir),
            FileDescriptor::Abstract(inode) => inode.create(name, read, write, isdir),
        }
    }
    fn kstat(&self, stat: &mut super::Kstat) {
        match self {
            FileDescriptor::File(inode) => inode.kstat(stat),
            FileDescriptor::Abstract(_) => {},
        }
    }
}
