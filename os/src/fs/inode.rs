use crate::fatfs::file::Inode;
use crate::fatfs::io::SeekFrom;
use crate::fatfs::root_dir;
use crate::fs::File;
use crate::memory::page_table::UserBuffer;
use crate::sync::UPsafeCell;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPsafeCell<OSInodeInner>,
}
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<UPsafeCell<Inode>>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<UPsafeCell<Inode>>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPsafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }

    pub fn read_all(&self) -> Vec<u8> {
        let inner = self.inner.exclusive_access();
        let offset = inner.offset;
        let v = inner.inode.inner.borrow_mut().read_all(offset);
        v
    }

    pub fn set_offset(&self, offset: usize) {
        self.inner.exclusive_access().offset = offset;
    }

    #[allow(unused)]
    pub fn get_offset(&self) -> usize {
        self.inner.exclusive_access().offset
    }
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0x000;
        const WRONLY = 0x001;
        const RDWR = 0x002;
        const CREATE = 0x40;
        const TRUNC = 1 << 10;
        const DIRECTORY = 0x0200000;
        const DIR = 0x040000;
        const FILE = 0x100000;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

pub fn root() -> Arc<OSInode> {
    Arc::new(OSInode::new(
        true,
        true,
        Arc::new(unsafe { UPsafeCell::new(root_dir()) }),
    ))
}

pub fn open_file(path: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    let isdir=flags.contains(OpenFlags::DIRECTORY);
    if flags.contains(OpenFlags::CREATE) {
        let file = root_dir().create(path, false).unwrap();
        Some(Arc::new(OSInode::new(
            readable,
            writable,
            Arc::new(unsafe { UPsafeCell::new(file) }),
        )))
    } else {
        if let Some(file) = root_dir().open(path, isdir) {
            print!("");
            Some(Arc::new(OSInode::new(
                readable,
                writable,
                Arc::new(unsafe { UPsafeCell::new(file) }),
            )))
        } else {
            None
        }
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }

    fn open(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        let ier = self.inner.exclusive_access();
        let mut inner = ier.inode.exclusive_access();
        if let Some(inode) = inner.open(name, isdir) {
            let os_inode = OSInode::new(read, write, Arc::new(unsafe { UPsafeCell::new(inode) }));
            Some(Arc::new(os_inode))
        } else {
            None
        }
    }

    fn seek(&self, offset: SeekFrom) -> usize {
        let mut inner = self.inner.exclusive_access();
        let offset = inner.inode.exclusive_access().seek(offset);
        inner.offset = offset;
        offset
    }

    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let offset = inner.offset;
            let len = inner.inode.exclusive_access().read(offset, *slice);
            if len == 0 {
                break;
            }
            inner.offset += len;
            total_read_size += len;
        }
        total_read_size
    }
    fn write(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();

        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let offset = inner.offset;
            let len = inner.inode.inner.borrow_mut().write(offset, *slice);
            if len == 0 {
                break;
            }
            inner.offset += len;
            total_write_size += len;
        }
        total_write_size
    }
    fn create(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        let inner = self.inner.exclusive_access();
        let mut inner = inner.inode.exclusive_access();
        if let Some(inode) = inner.create(name, isdir) {
            let os_inode = OSInode::new(read, write, Arc::new(unsafe { UPsafeCell::new(inode) }));
            Some(Arc::new(os_inode))
        } else {
            None
        }
    }

    fn kstat(&self, stat: &mut Kstat) {
        self.inner
            .exclusive_access()
            .inode
            .inner
            .borrow_mut()
            .stat(stat)
    }

    fn remove(&self, path: &str) -> bool {
        self.inner
            .exclusive_access()
            .inode
            .inner
            .borrow_mut()
            .remove(path)
    }

    fn name(&self) -> String {
        self.inner
            .inner
            .borrow_mut()
            .inode
            .inner
            .borrow_mut()
            .file_name()
    }
    fn getdents(&self, dirent: &mut Dirent) -> isize {
        self.inner
            .exclusive_access()
            .inode
            .exclusive_access()
            .getdents(dirent)

    }
}

#[repr(C)]
#[derive(Debug,Default)]
//文件状态信息结构
pub struct Kstat {
    pub st_dev: u64,
    pub sd_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub __pad: u64,
    pub st_size: i64,
    pub st_blksize: u32,
    pub __pad2: i32,
    pub st_blocks: u64,
    pub st_atime_sec: i64,
    pub st_atime_nsec: i64,
    pub st_mtime_sec: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime_sec: i64,
    pub st_ctime_nsec: i64,
    pub __unused: [i32; 2],
}

impl Kstat {
    pub fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Self>();
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as usize as *const u8,
                size,
            )
        }
    }
}

#[repr(C)]
pub struct Dirent {
    pub d_ino: usize,
    pub d_off: isize,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8;0]
}
