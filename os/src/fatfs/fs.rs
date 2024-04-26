use crate::sync::UPsafeCell;
use alloc::sync::Arc;
use core::convert::TryFrom;
use core::{borrow::BorrowMut, cmp, marker::PhantomData};
use log::{error, warn};

use super::{
    boot_sector::{BiosParameterBlock, BootSector},
    io::{Error, IoBase, Read, ReadLeExt, ReadWriteSeek, Seek, SeekFrom, Write},
    sdcard::BlockCacheManager,
    table::table_alloc_cluster,
};

pub struct FileSystem<IO: ReadWriteSeek> {
    pub disk: Arc<UPsafeCell<IO>>,
    pub bpb: BiosParameterBlock,
    pub root_dir_sectors: u32,
    pub total_clusters: u32,
    pub first_data_sector: u32,
    pub fs_info: UPsafeCell<FsInfoSector>,
}

pub trait IntoStorage<T: Read + Write + Seek> {
    fn into_storage(self) -> T;
}

impl<T: Read + Write + Seek> IntoStorage<T> for T {
    fn into_storage(self) -> Self {
        self
    }
}

impl<IO: Read + Write + Seek> FileSystem<IO> {
    pub fn new<T: IntoStorage<IO>>(storage: T) -> Result<Self, Error<IO::Error>> {
        let mut disk = storage.into_storage();
        assert_eq!(disk.seek(SeekFrom::Current(0))?, 0);
        let bpb = {
            let boot = BootSector::deserialize(&mut disk)?;
            boot.bpb
        };
        let root_dir_sectors = bpb.root_dir_sectors();
        let first_data_sector = bpb.first_data_sector();
        let total_clusters = bpb.total_clusters();
        disk.seek(SeekFrom::Start(
            bpb.bytes_from_sectors(bpb.fs_info_sector()),
        ))?;
        let fs_info = FsInfoSector::deserialize(&mut disk)?;
        unsafe {
            Ok(Self {
                disk: Arc::new(UPsafeCell::new(disk)),
                root_dir_sectors,
                first_data_sector,
                total_clusters,
                fs_info: UPsafeCell::new(fs_info),
                bpb,
            })
        }
    }

    pub fn byte_offset(&self, clusters: u32) -> u64 {
        self.bpb.bytes_from_sectors(
            self.bpb.sectors_from_clusters(clusters - 2) + self.first_data_sector,
        )
    }

    pub fn cluster_size(&self) -> u32 {
        self.bpb.cluster_size()
    }

    pub fn alloc_cluster(
        &self,
        prev_cluster: Option<u32>,
        zero: bool,
    ) -> Result<u32, Error<IO::Error>> {
        let hint = self.fs_info.inner.borrow_mut().next_free_cluster;

        let cluster = {
            let mut fat = self.fat_slice();
            match table_alloc_cluster(&mut fat, prev_cluster, hint, self.total_clusters) {
                Ok(x) => x,
                Err(_) => return Err(Error::AlreadyExists),
            }
        };
        if zero {
            let mut disk = self.disk.inner.borrow_mut();
            disk.seek(SeekFrom::Start(self.offset_from_cluster(cluster)))?;
            write_zeros(&mut *disk, u64::from(self.cluster_size()))?;
        }
        let mut fs_info = self.fs_info.inner.borrow_mut();
        fs_info.set_next_free_cluster(cluster + 1);
        fs_info.map_free_clusters(|n| n - 1);
        Ok(cluster)
    }

    pub fn fat_slice(&self) -> impl ReadWriteSeek<Error = Error<()>> + '_ {
        let sectors_per_fat = self.bpb.sectors_per_fat();
        let mirroring_enabled = self.bpb.mirroring_enabled();
        let (fat_first_sector, mirrors) = if mirroring_enabled {
            (self.bpb.reserved_sectors(), self.bpb.fats)
        } else {
            let active_fat = u32::from(self.bpb.active_fat());
            let fat_first_sector = (self.bpb.reserved_sectors()) + active_fat * sectors_per_fat;
            (fat_first_sector, 1)
        };
        let io = BlockCacheManager::new();
        DiskSlice::from_sectors(fat_first_sector, sectors_per_fat, mirrors, &self.bpb, io)
    }
    pub fn offset_from_cluster(&self, cluser: u32) -> u64 {
        self.offset_from_sector(self.sector_from_cluster(cluser))
    }
    fn offset_from_sector(&self, sector: u32) -> u64 {
        self.bpb.bytes_from_sectors(sector)
    }
    fn sector_from_cluster(&self, cluster: u32) -> u32 {
        self.first_data_sector + self.bpb.sectors_from_clusters(cluster - 2)
    }
}

fn fat_slice<S: ReadWriteSeek, B: BorrowMut<S>>(
    io: B,
    bpb: &BiosParameterBlock,
) -> impl ReadWriteSeek<Error = Error<S::Error>> {
    let sectors_per_fat = bpb.sectors_per_fat();
    let mirroring_enabled = bpb.mirroring_enabled();
    let (fat_first_sector, mirrors) = if mirroring_enabled {
        (bpb.reserved_sectors(), bpb.fats)
    } else {
        let active_fat = u32::from(bpb.active_fat());
        let fat_first_sector = (bpb.reserved_sectors()) + active_fat * sectors_per_fat;
        (fat_first_sector, 1)
    };
    DiskSlice::from_sectors(fat_first_sector, sectors_per_fat, mirrors, bpb, io)
}

#[derive(Clone, Default, Debug)]
pub struct FsInfoSector {
    free_cluster_count: Option<u32>,
    next_free_cluster: Option<u32>,
    dirty: bool,
}

impl FsInfoSector {
    const LEAD_SIG: u32 = 0x4161_5252;
    const STRUC_SIG: u32 = 0x6141_7272;
    const TRAIL_SIG: u32 = 0xAA55_0000;

    fn deserialize<R: Read>(rdr: &mut R) -> Result<Self, Error<R::Error>> {
        let lead_sig = rdr.read_u32_le()?;
        if lead_sig != Self::LEAD_SIG {
            error!("invalid lead_sig in FsInfo sector: {}", lead_sig);
            return Err(Error::CorruptedFileSystem);
        }
        let mut reserved = [0_u8; 480];
        rdr.read_exact(&mut reserved)?;
        let struc_sig = rdr.read_u32_le()?;
        if struc_sig != Self::STRUC_SIG {
            error!("invalid struc_sig in FsInfo sector: {}", struc_sig);
            return Err(Error::CorruptedFileSystem);
        }
        let free_cluster_count = match rdr.read_u32_le()? {
            0xFFFF_FFFF => None,
            // Note: value is validated in FileSystem::new function using values from BPB
            n => Some(n),
        };
        let next_free_cluster = match rdr.read_u32_le()? {
            0xFFFF_FFFF => None,
            0 | 1 => {
                warn!("invalid next_free_cluster in FsInfo sector (values 0 and 1 are reserved)");
                None
            }
            // Note: other values are validated in FileSystem::new function using values from BPB
            n => Some(n),
        };
        let mut reserved2 = [0_u8; 12];
        rdr.read_exact(&mut reserved2)?;
        let trail_sig = rdr.read_u32_le()?;
        if trail_sig != Self::TRAIL_SIG {
            error!("invalid trail_sig in FsInfo sector: {}", trail_sig);
            return Err(Error::CorruptedFileSystem);
        }
        Ok(Self {
            free_cluster_count,
            next_free_cluster,
            dirty: false,
        })
    }

    fn set_next_free_cluster(&mut self, cluster: u32) {
        self.next_free_cluster = Some(cluster);
        self.dirty = true;
    }
    fn set_free_cluster_count(&mut self, free_cluster_count: u32) {
        self.free_cluster_count = Some(free_cluster_count);
        self.dirty = true;
    }
    fn map_free_clusters(&mut self, map_fn: impl Fn(u32) -> u32) {
        if let Some(n) = self.free_cluster_count {
            self.free_cluster_count = Some(map_fn(n));
            self.dirty = true;
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FatType {
    /// 12 bits per FAT entry
    Fat12,
    /// 16 bits per FAT entry
    Fat16,
    /// 32 bits per FAT entry
    Fat32,
}

impl FatType {
    const FAT16_MIN_CLUSTERS: u32 = 4085;
    const FAT32_MIN_CLUSTERS: u32 = 65525;
    const FAT32_MAX_CLUSTERS: u32 = 0x0FFF_FFF4;

    pub fn from_clusters(total_clusters: u32) -> Self {
        if total_clusters < Self::FAT16_MIN_CLUSTERS {
            FatType::Fat12
        } else if total_clusters < Self::FAT32_MIN_CLUSTERS {
            FatType::Fat16
        } else {
            FatType::Fat32
        }
    }

    pub fn bits_per_fat_entry(self) -> u32 {
        match self {
            FatType::Fat12 => 12,
            FatType::Fat16 => 16,
            FatType::Fat32 => 32,
        }
    }

    pub fn min_clusters(self) -> u32 {
        match self {
            FatType::Fat12 => 0,
            FatType::Fat16 => Self::FAT16_MIN_CLUSTERS,
            FatType::Fat32 => Self::FAT32_MIN_CLUSTERS,
        }
    }

    pub fn max_clusters(self) -> u32 {
        match self {
            FatType::Fat12 => Self::FAT16_MIN_CLUSTERS - 1,
            FatType::Fat16 => Self::FAT32_MIN_CLUSTERS - 1,
            FatType::Fat32 => Self::FAT32_MAX_CLUSTERS,
        }
    }
}

/// A FAT volume status flags retrived from the Boot Sector and the allocation table second entry.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct FsStatusFlags {
    pub dirty: bool,
    pub io_error: bool,
}

impl FsStatusFlags {
    /// Checks if the volume is marked as dirty.
    ///
    /// Dirty flag means volume has been suddenly ejected from filesystem without unmounting.
    #[must_use]
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Checks if the volume has the IO Error flag active.
    #[must_use]
    pub fn io_error(&self) -> bool {
        self.io_error
    }

    fn encode(self) -> u8 {
        let mut res = 0_u8;
        if self.dirty {
            res |= 1;
        }
        if self.io_error {
            res |= 2;
        }
        res
    }

    pub fn decode(flags: u8) -> Self {
        Self {
            dirty: flags & 1 != 0,
            io_error: flags & 2 != 0,
        }
    }
}

pub struct DiskSlice<B, S = B> {
    begin: u64,
    size: u64,
    offset: u64,
    mirrors: u8,
    inner: B,
    phantom: PhantomData<S>,
}
impl<B: BorrowMut<S>, S: ReadWriteSeek> DiskSlice<B, S> {
    pub fn new(begin: u64, size: u64, mirrors: u8, inner: B) -> Self {
        Self {
            begin,
            size,
            mirrors,
            inner,
            offset: 0,
            phantom: PhantomData,
        }
    }

    fn from_sectors(
        first_sector: u32,
        sector_count: u32,
        mirrors: u8,
        bpb: &BiosParameterBlock,
        inner: B,
    ) -> Self {
        Self::new(
            bpb.bytes_from_sectors(first_sector),
            bpb.bytes_from_sectors(sector_count),
            mirrors,
            inner,
        )
    }

    pub fn abs_pos(&self) -> u64 {
        self.begin + self.offset
    }
}

// Note: derive cannot be used because of invalid bounds. See: https://github.com/rust-lang/rust/issues/26925
impl<B: Clone, S> Clone for DiskSlice<B, S> {
    fn clone(&self) -> Self {
        Self {
            begin: self.begin,
            size: self.size,
            offset: self.offset,
            mirrors: self.mirrors,
            inner: self.inner.clone(),
            // phantom is needed to add type bounds on the storage type
            phantom: PhantomData,
        }
    }
}

impl<B, S: IoBase> IoBase for DiskSlice<B, S> {
    type Error = Error<S::Error>;
}

impl<B: BorrowMut<S>, S: Read + Seek> Read for DiskSlice<B, S> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let offset = self.begin + self.offset;
        let read_size = cmp::min(self.size - self.offset, buf.len() as u64) as usize;
        let a = self.inner.borrow_mut().seek(SeekFrom::Start(offset))?;
        let size = self.inner.borrow_mut().read(&mut buf[..read_size])?;
        self.offset += size as u64;
        Ok(size)
    }
}

impl<B: BorrowMut<S>, S: Write + Seek> Write for DiskSlice<B, S> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let offset = self.begin + self.offset;
        let write_size = cmp::min(self.size - self.offset, buf.len() as u64) as usize;
        if write_size == 0 {
            return Ok(0);
        }
        // Write data
        let storage = self.inner.borrow_mut();
        for i in 0..self.mirrors {
            storage.seek(SeekFrom::Start(offset + u64::from(i) * self.size))?;
            storage.write_all(&buf[..write_size])?;
        }
        self.offset += write_size as u64;
        Ok(write_size)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(self.inner.borrow_mut().flush()?)
    }
}

impl<B, S: IoBase> Seek for DiskSlice<B, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let new_offset_opt: Option<u64> = match pos {
            SeekFrom::Current(x) => i64::try_from(self.offset)
                .ok()
                .and_then(|n| n.checked_add(x))
                .and_then(|n| u64::try_from(n).ok()),
            SeekFrom::Start(x) => Some(x),
            SeekFrom::End(o) => i64::try_from(self.size)
                .ok()
                .and_then(|size| size.checked_add(o))
                .and_then(|n| u64::try_from(n).ok()),
        };
        if let Some(new_offset) = new_offset_opt {
            if new_offset > self.size {
                error!("Seek beyond the end of the file");
                Err(Error::InvalidInput)
            } else {
                self.offset = new_offset;
                Ok(self.offset)
            }
        } else {
            error!("Invalid seek offset");
            Err(Error::InvalidInput)
        }
    }
}

pub fn write_zeros<IO: ReadWriteSeek>(disk: &mut IO, mut len: u64) -> Result<(), IO::Error> {
    const ZEROS: [u8; 512] = [0_u8; 512];
    while len > 0 {
        let write_size = cmp::min(len, ZEROS.len() as u64) as usize;
        disk.write_all(&ZEROS[..write_size])?;
        len -= write_size as u64;
    }
    Ok(())
}

fn write_zeros_until_end_of_sector<IO: ReadWriteSeek>(
    disk: &mut IO,
    bytes_per_sector: u16,
) -> Result<(), IO::Error> {
    let pos = disk.seek(SeekFrom::Current(0))?;
    let total_bytes_to_write = u64::from(bytes_per_sector) - (pos % u64::from(bytes_per_sector));
    if total_bytes_to_write != u64::from(bytes_per_sector) {
        write_zeros(disk, total_bytes_to_write)?;
    }
    Ok(())
}
