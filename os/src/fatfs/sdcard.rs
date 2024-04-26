use super::io::{IoBase, Read, Seek, SeekFrom, Write};
use crate::drivers::BlockDevice;
use crate::drivers::BLOCK_DEVICE;
use crate::sync::UPsafeCell;
use alloc::collections::BTreeMap;
use alloc::{collections::VecDeque, sync::Arc};
use core::cmp::min;
use core::convert::TryFrom;
use k210_pac::dmac::id;
use log::warn;
lazy_static::lazy_static!(
    pub static ref BLK_MANAGER: Arc<UPsafeCell<BlkManager>> = Arc::new(unsafe{UPsafeCell::new(BlkManager::new())});
);
#[derive(Debug)]
pub struct BlockCache {
    pub pos: usize,
    pub block_id: usize,
    pub dirty: bool,
    pub cache: [u8; 512],
}

unsafe impl Sync for BlockCache {}
unsafe impl Send for BlockCache {}

impl IoBase for BlockCache {
    type Error = ();
}

impl Read for BlockCache {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let min = min(512 - self.pos, buf.len());
        for i in 0..min {
            buf[i] = self.cache[self.pos + i];
        }
        self.pos += min;
        Ok(min)
    }
}

impl Write for BlockCache {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let min = min(512 - self.pos, buf.len());
        for i in 0..min {
            self.cache[self.pos + i] = buf[i];
        }
        BLOCK_DEVICE.write_block(self.block_id, &mut self.cache);
        self.pos += min;
        Ok(min)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Seek for BlockCache {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        match pos {
            SeekFrom::Start(x) => self.pos = x as usize,
            _ => panic!("seek error"),
        }
        Ok(1)
    }
}

pub struct BlockCacheManager {
    pub pos: usize,
    pub start: usize,
    pub size: usize,
    pub block_driver: Arc<dyn BlockDevice>,
}

impl BlockCacheManager {
    pub fn new() -> Self {
        BlockCacheManager {
            pos: 0,
            start: 0,
            size: 0,
            block_driver: BLOCK_DEVICE.clone(),
        }
    }
    pub fn from(start: usize, size: usize) -> Self {
        BlockCacheManager {
            pos: 0,
            size,
            start,
            block_driver: BLOCK_DEVICE.clone(),
        }
    }
}

impl IoBase for BlockCacheManager {
    type Error = ();
}

impl Read for BlockCacheManager {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut blk_manager = BLK_MANAGER.exclusive_access();
        let start_pos = self.pos;
        while !buf.is_empty() {
            let offset = self.pos % 512;
            let blk_id = self.pos / 512;
            let n = blk_manager.read_block(blk_id, &mut buf, &|blk, buf| {
                let len = min(buf.len(), 512 - offset);
                for idx in 0..len {
                    buf[idx] = blk.cache[offset + idx];
                }
                len
            });
            let tmp = buf;
            buf = &mut tmp[n..];
            self.pos += n;
        }
        Ok(self.pos - start_pos)
    }
}

impl Write for BlockCacheManager {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, Self::Error> {
        let mut blk_manager = BLK_MANAGER.exclusive_access();
        let start_pos = self.pos;
        while !buf.is_empty() {
            let offset = self.pos % 512;
            let blk_id = self.pos / 512;
            let n = blk_manager.write_block(blk_id, &mut buf, &|blk, buf| {
                let len = min(buf.len(), 512 - offset);
                for idx in 0..len {
                    blk.cache[offset + idx] = buf[idx];
                }
                len
            });
            let tmp = buf;
            buf = &tmp[n..];
            self.pos += n;
        }
        Ok(self.pos - start_pos)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        let mut blk_manager = BLK_MANAGER.exclusive_access();
        let start = self.start / 512;
        let end = (self.start + self.size) / 512;
        for blk in start..end {
            blk_manager.sync_block(blk);
        }
        Ok(())
    }
}

impl Seek for BlockCacheManager {
    fn seek(&mut self, pos: super::io::SeekFrom) -> Result<u64, Self::Error> {
        let new_offset_opt = match pos {
            SeekFrom::Start(x) => x as u32,
            SeekFrom::Current(x) => (self.pos as i64 + x) as u32,
            SeekFrom::End(_) => panic!("Seek Error"),
        };

        self.pos = new_offset_opt as usize;
        Ok(self.pos as u64)
    }
}

pub struct BlkManager {
    driver: Arc<dyn BlockDevice>,
    pub blocks: BTreeMap<usize, BlockCache>,
}
impl BlkManager {
    pub fn new() -> Self {
        Self {
            driver: BLOCK_DEVICE.clone(),
            blocks: BTreeMap::new(),
        }
    }

    pub fn read_block_from_disk(&mut self, blk_id: usize) {
        if self.blocks.len() > 100 {
            self.blocks.clear();
        }

        let mut blk = BlockCache {
            pos: 0,
            block_id: blk_id,
            dirty: false,
            cache: [0; 512],
        };
        self.driver.read_block(blk_id, &mut blk.cache);
        self.blocks.insert(blk_id, blk);
    }

    pub fn write_block_to_disk(&mut self, blk_id: usize) {
        if let Some(blk) = self.blocks.get(&blk_id) {
            self.driver.write_block(blk_id, &blk.cache);
        }
    }

    pub fn sync_block(&mut self, blk_id: usize) {
        self.blocks.remove(&blk_id);
    }

    pub fn read_block(
        &mut self,
        blk_id: usize,
        buf: &mut [u8],
        func: &dyn Fn(&BlockCache, &mut [u8]) -> usize,
    ) -> usize {
        if self.blocks.contains_key(&blk_id) {
            let blk = self.blocks.get_mut(&blk_id).unwrap();
            func.call_once((blk, buf))
        } else {
            self.read_block_from_disk(blk_id);
            let blk = self.blocks.get_mut(&blk_id).unwrap();
            func.call_once((blk, buf))
        }
    }

    pub fn write_block(
        &mut self,
        blk_id: usize,
        buf: &[u8],
        func: &dyn Fn(&mut BlockCache, &[u8]) -> usize,
    ) -> usize {
        if !self.blocks.contains_key(&blk_id) {
            self.read_block_from_disk(blk_id);
        };
        let len = if let Some(blk) = self.blocks.get_mut(&blk_id) {
            let len = func.call_once((blk, buf));

            blk.dirty = true;
            len
        } else {
            return 0;
        };
        self.write_block_to_disk(blk_id);
        len
    }
}
