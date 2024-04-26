use crate::{
    fs::{Dirent, File, Kstat},
    sync::UPsafeCell,
};
use alloc::{string::String, vec::Vec};
use core::{cmp, ptr::NonNull};
use k210_pac::{aes::en, wdt0::cr};
const MAX_FILE_SIZE: u32 = core::u32::MAX;
use super::{
    alloc_cluster, cluster_to_offset,
    dir_entry::{DirEntry, DirEntryEditor, DirFileEntry},
    io::{Error, IoBase, Read, Seek, SeekFrom, Write},
    sdcard::BlockCacheManager,
    FATFS,
};

pub struct FileEntry {
    pub name: String,
    pub first_cluster: Option<u32>,
    pub current_cluster: Option<u32>,
    pub pos: u64,
    pub entry: DirEntryEditor,
    pub disk: UPsafeCell<BlockCacheManager>,
    pub range: (u64, u64),
}

impl FileEntry {
    pub fn from(entry: DirFileEntry, entry_pos: u64, name: String) -> Self {
        let pos = 0;
        let abs_start_pos = match entry.first_cluster() {
            Some(x) => cluster_to_offset(x),
            None => 0,
        };
        let range = (abs_start_pos, abs_start_pos + entry.size());
        let first_cluster = None;
        let current_cluster = None;
        let mut disk = BlockCacheManager::from(abs_start_pos as usize, entry.size() as usize);
        disk.seek(SeekFrom::Start(abs_start_pos)).unwrap();
        let disk = unsafe { UPsafeCell::new(disk) };
        let entry = DirEntryEditor::new(entry, entry_pos);
        Self {
            pos,
            entry,
            disk,
            range,
            first_cluster,
            current_cluster,
            name,
        }
    }
    pub fn size(&self) -> u64 {
        self.entry.data.size()
    }
    pub fn set_first_cluster(&mut self, cluster: u32) {
        self.first_cluster = Some(cluster);
        self.entry.inner().set_first_cluster(self.first_cluster);
    }
    pub fn update_dir_entry_after_write(&mut self) {
        self.entry.set_size(self.pos as u32);
    }
    pub fn stat(&self, stat: &mut Kstat) {
        stat.st_dev = 1;
        stat.sd_ino = 1;
        stat.st_mode = 100000;
        stat.st_nlink = 1;
        stat.st_uid = 1;
        stat.st_gid = 1;
        stat.st_rdev = 1;
        stat.st_size = self.size() as i64;
        stat.st_blksize = 512;
        stat.st_blocks = 0;
        let access = self.entry.data.accessed();
        let create = self.entry.data.created();
        let modify = self.entry.data.modified();
        stat.st_atime_sec = 0;
        stat.st_atime_nsec = 0;
        stat.st_mtime_sec = modify.time.sec() as i64;
        stat.st_mtime_nsec = modify.time.msec() as i64 * 1000 ;
        stat.st_ctime_sec = create.time.sec() as i64;
        stat.st_ctime_nsec = create.time.msec() as i64 * 1000;
    }
}

impl IoBase for FileEntry {
    type Error = ();
}

impl Seek for FileEntry {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let disk_offset = match pos {
            SeekFrom::Start(x) => {
                let offset = self.range.0 + x;
                self.pos = x as u64;
                offset
            }
            SeekFrom::End(x) => {
                let offset = self.range.1 as i64 + x;
                self.pos = x as u64;
                offset as u64
            }
            SeekFrom::Current(x) => {
                let offset = x + self.pos as i64;
                self.range.0 + offset as u64
            }
        };
        self.disk
            .inner
            .borrow_mut()
            .seek(SeekFrom::Start(disk_offset))
            .unwrap();
        Ok(self.pos)
    }
}

impl Read for FileEntry {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut disk = self.disk.inner.borrow_mut();
        let off = self.size() - self.pos;
        disk.seek(SeekFrom::Start(
            cluster_to_offset(self.entry.inner().first_cluster().unwrap()) + self.pos,
        ))
            .unwrap();
        match disk.read(buf) {
            Ok(len) => {
                let len = len as u64;
                if off <= len {
                    self.pos += off;
                    Ok(off as usize)
                } else {
                    self.pos += len;
                    Ok(len as usize)
                }
            }
            Err(x) => Err(x),
        }
    }
}

impl Write for FileEntry {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let cluster_size = FATFS.cluster_size() as u64;
        let offset_in_cluster = self.pos % cluster_size;
        let bytes_left_in_cluster = (cluster_size - offset_in_cluster) as usize;
        let bytes_left_until_max_file_size = (MAX_FILE_SIZE - self.pos as u32) as usize;
        let write_size = cmp::min(buf.len(), bytes_left_in_cluster);
        let write_size = cmp::min(write_size, bytes_left_until_max_file_size);
        if write_size == 0 {
            return Ok(0);
        }
        let current_cluster = if self.pos % cluster_size == 0 {
            let next_cluster = match self.current_cluster {
                Some(n) => return Err(()),
                None => self.first_cluster,
            };
            if let Some(n) = next_cluster {
                n
            } else {
                let new_cluster = alloc_cluster(self.current_cluster, false).unwrap();
                if self.first_cluster.is_none() {
                    self.set_first_cluster(new_cluster);
                }
                new_cluster
            }
        } else {
            match self.current_cluster {
                Some(n) => n,
                None => panic!("Offset inside cluster but no cluster allocated"),
            }
        };
        let offset_in_fs = cluster_to_offset(current_cluster) + u64::from(offset_in_cluster);
        let written_bytes = {
            let mut disk = BlockCacheManager::new();
            disk.seek(SeekFrom::Start(offset_in_fs))?;
            disk.write(&buf[..write_size])?
        };
        if written_bytes == 0 {
            return Ok(0);
        }
        self.pos += written_bytes as u64;
        self.current_cluster = Some(current_cluster);
        self.update_dir_entry_after_write();
        self.entry.flush();
        Ok(written_bytes)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.disk.exclusive_access().flush()
    }
}

pub enum Inode {
    File(FileEntry),
    Dir(DirEntry),
}

impl Inode {
    pub fn ls(&mut self) {
        match self {
            Inode::File(_) => {}
            Inode::Dir(dir) => dir.ls(),
        }
    }

    pub fn is_dir(&self) -> bool {
        match self {
            Inode::File(_) => false,
            Inode::Dir(dir) => dir.is_dir(),
        }
    }
    pub fn is_file(&self) -> bool {
        match self {
            Inode::File(_) => true,
            Inode::Dir(dir) => !dir.is_dir(),
        }
    }

    pub fn create(&mut self, name: &str, isdir: bool) -> Option<Inode> {
        match self {
            Inode::File(_) => {}
            Inode::Dir(dir) => match isdir {
                true => {
                    if let Ok(file) = dir.create_dir(name) {
                        return Some(file);
                    }
                }
                false => {

                    if let Ok(file) = dir.create_file(name) {
                        return Some(file);
                    }
                }
            },
        }
        None
    }

    pub fn open(&mut self, name: &str, isdir: bool) -> Option<Inode> {
        match self {
            Inode::File(_) => {}
            Inode::Dir(dir) => match isdir {
                true => {
                    if let Ok(dir) = dir.open_dir(name) {
                        return Some(dir);
                    }
                }
                false => {
                    if let Ok(file) = dir.open_file(name) {
                        return Some(file);
                    }
                }
            },
        }
        None
    }

    pub fn read(&mut self, offset: usize, buf: &mut [u8]) -> usize {
        match self {
            Inode::File(file) => {
                file.seek(SeekFrom::Start(offset as u64)).unwrap();
                file.read(buf).unwrap()
            }
            Inode::Dir(_) => 0,
        }
    }

    pub fn write(&mut self, offset: usize, buf: &mut [u8]) -> usize {
        match self {
            Inode::File(file) => {
                file.seek(SeekFrom::Start(offset as u64)).unwrap();
                file.write(buf).unwrap()
            }
            Inode::Dir(_) => 0,
        }
    }

    pub fn read_all(&mut self, offset: usize) -> Vec<u8> {
        match self {
            Inode::File(file) => {
                let mut v = Vec::new();
                file.seek(SeekFrom::Start(offset as u64)).unwrap();
                file.read_to_end(&mut v).unwrap();
                v
            }
            Inode::Dir(dir) => panic!("Not File {} ", dir.file_name()),
        }
    }

    pub fn file_name(&self) -> String {
        match self {
            Inode::File(file) => String::clone(&file.name),
            Inode::Dir(dir) => dir.file_name(),
        }
    }
    pub fn seek(&mut self, offset: SeekFrom) -> usize {
        match self {
            Inode::File(file) => file.seek(offset).unwrap() as usize,
            Inode::Dir(dir) => 0,
        }
    }

    pub fn remove(&mut self, path: &str) -> bool {
        match self {
            Inode::File(file) => false,
            Inode::Dir(dir) => dir.remove(path).is_ok(),
        }
    }
    pub fn stat(&mut self, stat: &mut Kstat) {
        match self {
            Inode::File(file) => file.stat(stat),
            Inode::Dir(dir) => {}
        }
    }
    pub fn getdents(&mut self, dirent: &mut Dirent) -> isize {
        match self {
            Inode::File(_) => -1,
            Inode::Dir(dir) => {
                dir.seek( SeekFrom::Start(0)).unwrap();
                let current = dir.dirents;
                let mut nread = 0;
                dirent.d_ino = current as usize;
                dirent.d_off = 0;
                dirent.d_reclen = (dir.dirents - current) as u16;
                unsafe {
                    let d_name = core::slice::from_raw_parts_mut(dirent.d_name.as_ptr() as *mut u8, dir.file_name().len());
                    d_name.clone_from_slice(dir.file_name().as_bytes());
                }
                for e in dir {
                    nread+=1;
                }
                nread
            }
        }
    }
}

impl Drop for Inode {
    fn drop(&mut self) {
        match self {
            Inode::File(file) => file.flush().unwrap(),
            Inode::Dir(_) => {}
        }
        drop(self)
    }
}
