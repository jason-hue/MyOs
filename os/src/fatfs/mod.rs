#![allow(unused)]
pub mod boot_sector;
pub mod dir_entry;
pub mod file;
pub mod fs;
pub mod io;
pub mod lfn;
pub mod sdcard;
pub mod table;
pub mod time;
use alloc::{
    collections::BTreeMap,
    sync::Arc,
    vec::{self, Vec},
};
use fs::FileSystem;
use lazy_static::lazy_static;
use log::info;
use sdcard::BlockCacheManager;

use crate::{
    drivers::BLOCK_DEVICE,
    fatfs::io::{Read, Seek, SeekFrom, Write},
    fs::File,
};

use self::{
    dir_entry::DirEntry,
    file::{FileEntry, Inode},
    io::Error,
};
lazy_static! {
    pub static ref FATFS: Arc<FileSystem<BlockCacheManager>> =
        Arc::new(FileSystem::new(BlockCacheManager::new()).unwrap());
}

pub fn fs_init() {
    root_dir().ls();

}

#[inline]
pub fn alloc_cluster(prev_cluster: Option<u32>, zero: bool) -> Result<u32, Error<()>> {
    FATFS.alloc_cluster(prev_cluster, zero)
}

#[inline]
pub fn root_dir() -> Inode {
    Inode::Dir(DirEntry::root_dir(FATFS.bpb.root_dir_first_cluster))
}

#[inline]
pub fn cluster_to_offset(cluster: u32) -> u64 {
    FATFS.byte_offset(cluster) as u64
}
