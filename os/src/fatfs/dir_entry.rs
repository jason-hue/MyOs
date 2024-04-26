use alloc::string::String;
use bitflags::bitflags;
use core::{convert::TryInto, mem::size_of};
use k210_pac::aes::en;
use log::{error, info};

use super::{
    alloc_cluster, cluster_to_offset,
    file::{FileEntry, Inode},
    fs::DiskSlice,
    io::{self, Error, IoBase, IoError, Read, ReadLeExt, Seek, SeekFrom, Write, WriteLeExt},
    lfn::{
        char_to_uppercase, lfn_checksum, validate_long_name, LfnBuffer, LfnEntriesGenerator,
        ShortNameGenerator, DIR_ENTRY_DELETED_FLAG, DIR_ENTRY_SIZE, SFN_SIZE,
    },
    sdcard::BlockCacheManager,
    time::{get_current_date_time, Date, DateTime, Time},
    FATFS,
};

use crate::{
    fatfs::lfn::{LongNameBuilder, ShortName, LFN_PART_LEN},
    fs::Dirent,
    sync::UPsafeCell, console::print,
};

bitflags! {
    #[derive(Default)]
    pub struct DirAttr:u8{
        const READ_ONLY  = 0x01;
        const HIDDEN     = 0x02;
        const SYSTEM     = 0x04;
        const VOLUME_ID  = 0x08;
        const DIRECTORY  = 0x10;
        const ARCHIVE    = 0x20;
        const LFN        = Self::READ_ONLY.bits | Self::HIDDEN.bits
                         | Self::SYSTEM.bits | Self::VOLUME_ID.bits;
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct DirFileEntry {
    name: [u8; 11],
    attrs: DirAttr,
    reserved_0: u8,
    create_time_0: u8,
    create_time_1: u16,
    create_date: u16,
    access_date: u16,
    first_cluster_hi: u16,
    modify_time: u16,
    modify_date: u16,
    first_cluster_lo: u16,
    size: u32,
}

impl DirFileEntry {
    pub fn deserialize<R: Read>(rdr: &mut R) -> Result<Self, Error<R::Error>> {
        let mut name = [0; 11];
        rdr.read_exact(&mut name)?;
        let attrs = DirAttr::from_bits_truncate(rdr.read_u8()?);
        let data = DirFileEntry {
            name,
            attrs,
            reserved_0: rdr.read_u8()?,
            create_time_0: rdr.read_u8()?,
            create_time_1: rdr.read_u16_le()?,
            create_date: rdr.read_u16_le()?,
            access_date: rdr.read_u16_le()?,
            first_cluster_hi: rdr.read_u16_le()?,
            modify_time: rdr.read_u16_le()?,
            modify_date: rdr.read_u16_le()?,
            first_cluster_lo: rdr.read_u16_le()?,
            size: rdr.read_u32_le()?,
        };
        Ok(data)
    }

    pub fn serialize<W: Write>(&self, wrt: &mut W) -> Result<(), W::Error> {
        wrt.write_all(&self.name)?;
        wrt.write_u8(self.attrs.bits())?;
        wrt.write_u8(self.reserved_0)?;
        wrt.write_u8(self.create_time_0)?;
        wrt.write_u16_le(self.create_time_1)?;
        wrt.write_u16_le(self.create_date)?;
        wrt.write_u16_le(self.access_date)?;
        wrt.write_u16_le(self.first_cluster_hi)?;
        wrt.write_u16_le(self.modify_time)?;
        wrt.write_u16_le(self.modify_date)?;
        wrt.write_u16_le(self.first_cluster_lo)?;
        wrt.write_u32_le(self.size)?;
        Ok(())
    }

    pub fn is_dir(&self) -> bool {
        self.attrs.contains(DirAttr::DIRECTORY)
    }

    pub fn first_cluster(&self) -> Option<u32> {
        let n = (u32::from(self.first_cluster_hi) << 16) | u32::from(self.first_cluster_lo);
        if n == 0 {
            None
        } else {
            Some(n)
        }
    }

    pub fn new(name: [u8; SFN_SIZE], attrs: DirAttr) -> Self {
        Self {
            name,
            attrs,
            ..Self::default()
        }
    }

    pub fn set_first_cluster(&mut self, cluster: Option<u32>) {
        let n = cluster.unwrap_or(0);
        self.first_cluster_hi = (n >> 16) as u16;
        self.first_cluster_lo = (n & 0xFFFF) as u16;
    }

    pub fn renamed(&self, new_name: [u8; SFN_SIZE]) -> Self {
        let mut sfn_entry = self.clone();
        sfn_entry.name = new_name;
        sfn_entry
    }

    pub fn name(&self) -> &[u8; SFN_SIZE] {
        &self.name
    }

    pub fn is_end(&self) -> bool {
        self.name[0] == 0
    }

    pub fn size(&self) -> u64 {
        self.size as u64
    }
    pub fn is_deleted(&self) -> bool {
        self.name[0] == DIR_ENTRY_DELETED_FLAG
    }
    pub fn set_size(&mut self, size: u32) {
        if self.size != size {
            self.size = size
        }
    }

    fn lowercase_basename(&self) -> bool {
        self.reserved_0 & (1 << 3) != 0
    }
    fn lowercase_ext(&self) -> bool {
        self.reserved_0 & (1 << 4) != 0
    }
    fn lowercase_name(&self) -> ShortName {
        let mut name_copy: [u8; SFN_SIZE] = self.name;
        if self.lowercase_basename() {
            name_copy[..8].make_ascii_lowercase();
        }
        if self.lowercase_ext() {
            name_copy[8..].make_ascii_lowercase();
        }
        ShortName::new(&name_copy)
    }

    pub fn created(&self) -> DateTime {
        DateTime::decode(self.create_date, self.create_time_1, self.create_time_0)
    }

    pub fn accessed(&self) -> Date {
        Date::decode(self.access_date)
    }

    pub fn modified(&self) -> DateTime {
        DateTime::decode(self.modify_date, self.modify_time, 0)
    }

    pub(crate) fn set_created(&mut self, date_time: DateTime) {
        self.create_date = date_time.date.encode();
        let encoded_time = date_time.time.encode();
        self.create_time_1 = encoded_time.0;
        self.create_time_0 = encoded_time.1;
    }

    pub(crate) fn set_accessed(&mut self, date: Date) {
        self.access_date = date.encode();
    }

    pub(crate) fn set_modified(&mut self, date_time: DateTime) {
        self.modify_date = date_time.date.encode();
        self.modify_time = date_time.time.encode().0;
    }
    pub(crate) fn set_deleted(&mut self) {
        self.name[0] = DIR_ENTRY_DELETED_FLAG;
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DirLfnEntry {
    order: u8,
    name_0: [u16; 5],
    attrs: DirAttr,
    entry_type: u8,
    checksum: u8,
    name_1: [u16; 6],
    reserved_0: u16,
    name_2: [u16; 2],
}

impl DirLfnEntry {
    pub fn new(order: u8, checksum: u8) -> Self {
        Self {
            order,
            checksum,
            attrs: DirAttr::LFN,
            ..Self::default()
        }
    }
    pub fn serialize<W: Write>(&self, wrt: &mut W) -> Result<(), W::Error> {
        wrt.write_u8(self.order)?;
        for ch in &self.name_0 {
            wrt.write_u16_le(*ch)?;
        }
        wrt.write_u8(self.attrs.bits())?;
        wrt.write_u8(self.entry_type)?;
        wrt.write_u8(self.checksum)?;
        for ch in &self.name_1 {
            wrt.write_u16_le(*ch)?;
        }
        wrt.write_u16_le(self.reserved_0)?;
        for ch in &self.name_2 {
            wrt.write_u16_le(*ch)?;
        }
        Ok(())
    }

    pub fn copy_name_to_slice(&self, lfn_part: &mut [u16]) {
        debug_assert!(lfn_part.len() == LFN_PART_LEN);
        lfn_part[0..5].copy_from_slice(&self.name_0);
        lfn_part[5..11].copy_from_slice(&self.name_1);
        lfn_part[11..13].copy_from_slice(&self.name_2);
    }

    pub fn copy_name_from_slice(&mut self, lfn_part: &[u16; LFN_PART_LEN]) {
        self.name_0.copy_from_slice(&lfn_part[0..5]);
        self.name_1.copy_from_slice(&lfn_part[5..5 + 6]);
        self.name_2.copy_from_slice(&lfn_part[11..11 + 2]);
    }

    pub fn order(&self) -> u8 {
        self.order
    }

    pub fn checksum(&self) -> u8 {
        self.checksum
    }

    pub fn is_deleted(&self) -> bool {
        self.order == DIR_ENTRY_DELETED_FLAG
    }

    pub fn set_deleted(&mut self) {
        self.order = DIR_ENTRY_DELETED_FLAG;
    }

    pub fn is_end(&self) -> bool {
        self.order == 0
    }
}
#[derive(Debug)]
pub enum DirEntryData {
    File(DirFileEntry),
    Lfn(DirLfnEntry),
}

impl DirEntryData {
    pub fn is_deleted(&self) -> bool {
        match self {
            DirEntryData::File(file) => file.is_deleted(),
            DirEntryData::Lfn(lfn) => lfn.is_deleted(),
        }
    }

    pub(crate) fn serialize<W: Write>(&self, wrt: &mut W) -> Result<(), W::Error> {
        match self {
            DirEntryData::File(file) => file.serialize(wrt),
            DirEntryData::Lfn(lfn) => lfn.serialize(wrt),
        }
    }

    pub fn deserialize<R: Read>(rdr: &mut R) -> Result<Self, R::Error> {
        let mut name = [0; SFN_SIZE];
        rdr.read_exact(&mut name)?;
        let attrs = DirAttr::from_bits_truncate(rdr.read_u8()?);
        if attrs & DirAttr::LFN == DirAttr::LFN {
            let mut data = DirLfnEntry {
                attrs,
                ..DirLfnEntry::default()
            };
            data.order = name[0];
            for (dst, src) in data.name_0.iter_mut().zip(name[1..].chunks_exact(2)) {
                // unwrap cannot panic because src has exactly 2 values
                *dst = u16::from_le_bytes(src.try_into().unwrap());
            }
            data.entry_type = rdr.read_u8()?;
            data.checksum = rdr.read_u8()?;
            for x in &mut data.name_1 {
                *x = rdr.read_u16_le()?;
            }
            data.reserved_0 = rdr.read_u16_le()?;
            for x in &mut data.name_2 {
                *x = rdr.read_u16_le()?;
            }
            Ok(DirEntryData::Lfn(data))
        } else {
            let data = DirFileEntry {
                name,
                attrs,
                reserved_0: rdr.read_u8()?,
                create_time_0: rdr.read_u8()?,
                create_time_1: rdr.read_u16_le()?,
                create_date: rdr.read_u16_le()?,
                access_date: rdr.read_u16_le()?,
                first_cluster_hi: rdr.read_u16_le()?,
                modify_time: rdr.read_u16_le()?,
                modify_date: rdr.read_u16_le()?,
                first_cluster_lo: rdr.read_u16_le()?,
                size: rdr.read_u32_le()?,
            };
            Ok(DirEntryData::File(data))
        }
    }

    pub fn is_end(&self) -> bool {
        match self {
            DirEntryData::File(file) => file.is_end(),
            DirEntryData::Lfn(lfn) => lfn.is_end(),
        }
    }

    pub(crate) fn set_deleted(&mut self) {
        match self {
            DirEntryData::File(file) => file.set_deleted(),
            DirEntryData::Lfn(lfn) => lfn.set_deleted(),
        }
    }
}

pub struct DirEntry {
    pub dir_entry: DirFileEntry,
    pub offset: u64, // abs offset
    pub disk: UPsafeCell<BlockCacheManager>,
    pub short_name: ShortName,
    pub lfn_utf16: LfnBuffer,
    pub offset_range: (u64, u64),
    pub entry_pos: u64,
    pub dirents: u64,
}

impl DirEntry {
    pub fn root_dir(first_cluster: u32) -> Self {
        let mut blk = BlockCacheManager::new();
        let offset = cluster_to_offset(first_cluster);
        blk.seek(SeekFrom::Start(offset)).unwrap();
        let mut name = [0u8;11];
        name[0] = '/' as u8;
        let dir_entry = DirFileEntry {
            first_cluster_hi: ((first_cluster >> 16) & !(1 << 16)) as u16,
            first_cluster_lo: (first_cluster & !(1 << 16)) as u16,
            name,
            ..Default::default()
        };

        Self {
            dir_entry,
            disk: unsafe { UPsafeCell::new(blk) },
            offset,
            short_name: ShortName::new(b"/          "),
            lfn_utf16: LfnBuffer::new(),
            entry_pos: 0,
            offset_range: (0, 0),
            dirents: 0,
        }
    }
    pub fn new(first_cluster: u32) -> Self {
        let mut blk = BlockCacheManager::new();
        let offset = cluster_to_offset(first_cluster);
        blk.seek(SeekFrom::Start(offset)).unwrap();
        let dir_entry = DirFileEntry::deserialize(&mut blk).unwrap();
        Self {
            dir_entry,
            disk: unsafe { UPsafeCell::new(blk) },
            offset,
            short_name: ShortName::new(b"  root     "),
            lfn_utf16: LfnBuffer::new(),
            entry_pos: 0,
            offset_range: (0, 0),
            dirents: 0,
        }
    }
    pub fn long_file_name_as_ucs2_units(&self) -> Option<&[u16]> {
        if self.lfn_utf16.len() > 0 {
            Some(self.lfn_utf16.as_ucs2_units())
        } else {
            None
        }
    }
    pub fn file_name(&self) -> String {
        let lfn_opt = self.long_file_name_as_ucs2_units();
        if let Some(lfn) = lfn_opt {
            String::from_utf16_lossy(lfn)
        } else {
            self.dir_entry.lowercase_name().to_string()
        }
    }
    pub fn find_entry(&mut self, name: &str, is_dir: Option<bool>) -> Result<DirEntry, Error<()>> {
        self.seek(SeekFrom::Start(0)).unwrap();
        for e in self {

            if e.eq_name(name) {
                if is_dir.is_some() && Some(e.is_dir()) != is_dir {
                    if e.is_dir() {
                        error!("Is a directory");
                    } else {
                        error!("Not a directory");
                    }
                    return Err(Error::Io(()));
                }
                return Ok(e);
            }
        }
        Err(Error::NotFound)
    }

    pub fn eq_name(&self, name: &str) -> bool {
        if self.eq_name_lfn(name) {
            true
        } else {
            self.short_name.eq_ignore_case(name)
        }
    }

    fn create_sfn_entry(
        &self,
        short_name: [u8; SFN_SIZE],
        attrs: DirAttr,
        first_cluster: Option<u32>,
    ) -> DirFileEntry {
        let mut raw_entry = DirFileEntry::new(short_name, attrs);
        raw_entry.set_first_cluster(first_cluster);
        let now = get_current_date_time();
        raw_entry.set_created(now);
        raw_entry.set_accessed(now.date);
        raw_entry.set_modified(now);
        raw_entry
    }

    fn check_for_existence(
        &mut self,
        name: &str,
        is_dir: Option<bool>,
    ) -> Result<DirEntryOrShortName, Error<()>> {
        let mut short_name_gen = ShortNameGenerator::new(name);
        loop {
            match self.find_entry(name, is_dir) {
                Ok(e) => return Ok(DirEntryOrShortName::DirEntry(e)),
                Err(Error::NotFound) => {  }
                Err(e) => return Err(e),
            }
            if let Ok(name) = short_name_gen.generate() {
                return Ok(DirEntryOrShortName::ShortName(name));
            }
            short_name_gen.next_iteration();
        }
    }

    fn find_free_entries(&mut self, num_entries: u32) -> Result<BlockCacheManager, Error<()>> {
        let start = self.seek(SeekFrom::Start(0)).unwrap();
        let mut stream = self.disk.inner.borrow_mut();

        let mut first_free: u32 = 0;
        let mut num_free: u32 = 0;
        let mut i: u32 = 0;
        loop {
            let raw_entry = DirEntryData::deserialize(&mut *stream)?;

            if raw_entry.is_end() {
                // first unused entry - all remaining space can be used
                if num_free == 0 {
                    first_free = i;
                }
                let pos = u64::from(first_free * DIR_ENTRY_SIZE);

                let mut disk = BlockCacheManager::new();
                let offset = cluster_to_offset(self.dir_entry.first_cluster().unwrap()) + pos;
                disk.seek(SeekFrom::Start(offset)).unwrap();
                return Ok(disk);
            } else if raw_entry.is_deleted() {
                // free entry - calculate number of free entries in a row
                if num_free == 0 {
                    first_free = i;
                }
                num_free += 1;
                if num_free == num_entries {
                    // enough space for new file
                    let pos = u64::from(first_free * DIR_ENTRY_SIZE);
                    let mut disk = BlockCacheManager::new();

                    let offset = cluster_to_offset(self.dir_entry.first_cluster().unwrap()) + pos;
                    disk.seek(SeekFrom::Start(offset)).unwrap();
                    return Ok(disk);
                }
            } else {
                // used entry - start counting from 0
                num_free = 0;
            }
            i += 1;
        }
    }

    fn alloc_and_write_lfn_entries(
        &mut self,
        lfn_utf16: &LfnBuffer,
        short_name: &[u8; SFN_SIZE],
    ) -> Result<(BlockCacheManager, u64), Error<()>> {
        let lfn_chsum = lfn_checksum(short_name);
        let a = lfn_utf16.as_ucs2_units();
        let lfn_iter = LfnEntriesGenerator::new(lfn_utf16.as_ucs2_units(), lfn_chsum);
        let num_entries = lfn_iter.len() as u32 + 1;

        let mut stream = self.find_free_entries(num_entries)?;
        let start_pos = stream.seek(io::SeekFrom::Current(0))?;
        for lfn_entry in lfn_iter {
            lfn_entry.serialize(&mut stream)?;
        }
        Ok((stream, start_pos))
    }

    fn write_entry(&mut self, name: &str, raw_entry: DirFileEntry) -> Result<DirEntry, Error<()>> {
        validate_long_name(name)?;
        let lfn_utf16 = Self::encode_lfn_utf16(name);
        let (mut stream, start_pos) =
            self.alloc_and_write_lfn_entries(&lfn_utf16, raw_entry.name())?;
        raw_entry.serialize(&mut stream)?;
        let entry_pos = stream.pos as u64 - DIR_ENTRY_SIZE as u64;
        let end_pos = stream.seek(io::SeekFrom::Current(0))?;
        let short_name = ShortName::new(raw_entry.name());
        let offset = 0;
        let disk = unsafe { UPsafeCell::new(BlockCacheManager::new()) };
        let offset_range = (start_pos, end_pos);
        Ok(DirEntry {
            dir_entry: raw_entry,
            short_name,
            lfn_utf16,
            entry_pos,
            offset,
            disk,
            offset_range,
            dirents: 0,
        })
    }

    fn encode_lfn_utf16(name: &str) -> LfnBuffer {
        LfnBuffer::from_ucs2_units(name.encode_utf16())
    }

    pub fn create_file(&mut self, path: &str) -> Result<Inode, Error<()>> {

        let (name, rest_opt) = split_path(path);
        if let Some(rest) = rest_opt {
            return self.find_entry(name, Some(true))?.create_file(rest);
        }
        let e = self.check_for_existence(name, Some(false))?;
        match e {
            DirEntryOrShortName::DirEntry(e) => match e.is_dir() {
                true => Ok(Inode::Dir(e)),
                false => Ok(Inode::File(e.to_file())),
            },
            DirEntryOrShortName::ShortName(short_name) => {
                let sfn_entry =
                    self.create_sfn_entry(short_name, DirAttr::from_bits_truncate(0), None);
                let e = self.write_entry(name, sfn_entry)?;
                Ok(Inode::File(e.to_file()))
            }
        }
    }

    pub fn create_dir(&mut self, path: &str) -> Result<Inode, Error<()>> {
        let (name, rest_opt) = split_path(path);
        if let Some(rest) = rest_opt {
            return self.find_entry(name, Some(true))?.create_dir(rest);
        }

        let r = self.check_for_existence(name, Some(true))?;
        match r {
            DirEntryOrShortName::DirEntry(e) => Ok(Inode::Dir(e)),
            DirEntryOrShortName::ShortName(short_name) => {
                let cluster = alloc_cluster(None, true).unwrap();
                let sfn_entry =
                    self.create_sfn_entry(short_name, DirAttr::DIRECTORY, Some(cluster));
                let mut entry = self.write_entry(name, sfn_entry)?;

                let dot_sfn = ShortNameGenerator::generate_dot();
                let sfn_entry = self.create_sfn_entry(
                    dot_sfn,
                    DirAttr::DIRECTORY,
                    entry.dir_entry.first_cluster(),
                );
                entry.write_entry(".", sfn_entry)?;

                let dotdot_sfn = ShortNameGenerator::generate_dotdot();
                let sfn_entry = self.create_sfn_entry(
                    dotdot_sfn,
                    DirAttr::DIRECTORY,
                    self.dir_entry.first_cluster(),
                );
                entry.write_entry("..", sfn_entry)?;
                Ok(Inode::Dir(entry))
            }
        }
    }

    pub fn is_dir(&self) -> bool {
        self.dir_entry.is_dir()
    }

    fn eq_name_lfn(&self, name: &str) -> bool {
        if let Some(lfn) = self.long_file_name_as_ucs2_units() {
            let self_decode_iter = char::decode_utf16(lfn.iter().copied());
            let mut other_uppercase_iter = name.chars().flat_map(char_to_uppercase);
            for decode_result in self_decode_iter {
                if let Ok(self_char) = decode_result {
                    for self_uppercase_char in char_to_uppercase(self_char) {
                        // compare each character in uppercase
                        if Some(self_uppercase_char) != other_uppercase_iter.next() {
                            return false;
                        }
                    }
                } else {
                    // decoding failed
                    return false;
                }
            }
            // both iterators should be at the end here
            other_uppercase_iter.next() == None
        } else {
            // entry has no long name
            false
        }
    }
    //显示目录包含的目录以及文件
    pub fn ls(&mut self) {
        self.seek(SeekFrom::Current(0)).unwrap();
        for dir in self {
            if dir.is_dir() {
                println!("Dir: {}", dir.file_name());
            } else {
                println!("File: {}", dir.file_name());
            }
        }
    }

    pub fn read_next_entry(&mut self) -> Result<Option<DirEntry>, Error<()>> {

        let mut lfn_builder = LongNameBuilder::new();
        let mut offset = self.disk.inner.borrow_mut().seek(SeekFrom::Current(0))?;
        let mut begin_offset = offset;
        loop {
            let raw_entry = DirEntryData::deserialize(self)?;
            offset += u64::from(DIR_ENTRY_SIZE);
            if raw_entry.is_end() {
                self.seek(SeekFrom::Current(0))?;
                return Ok(None);
            }
            if raw_entry.is_deleted() {
                lfn_builder.clear();
                begin_offset = offset;
                continue;
            }
            match raw_entry {
                DirEntryData::File(data) => {
                    let entry_pos = self.disk.inner.borrow_mut().pos as u64 - DIR_ENTRY_SIZE as u64;
                    lfn_builder.validate_chksum(data.name());
                    let short_name = ShortName::new(data.name());
                    let mut blk = BlockCacheManager::new();

                    blk.seek(SeekFrom::Start(offset))?;
                    let disk = unsafe { UPsafeCell::new(blk) };
                    return Ok({
                        Some(Self {
                            dir_entry: data,
                            offset,
                            disk,
                            short_name,
                            entry_pos,
                            lfn_utf16: lfn_builder.into_buf(),
                            offset_range: (begin_offset, offset),
                            dirents: 0,
                        })
                    });
                }
                DirEntryData::Lfn(data) => {
                    lfn_builder.process(&data);
                }
            }
        }
    }

    pub fn open_file(&mut self, path: &str) -> Result<Inode, Error<()>> {
        let (name, rest_opt) = split_path(path);
        if let Some(rest) = rest_opt {
            let mut e = self.find_entry(name, Some(true))?;
            return e.open_file(rest);
        }
        let e = self.find_entry(name, Some(false))?;
        match e.is_dir() {
            true => Ok(Inode::Dir(e)),
            false => Ok(Inode::File(e.to_file())),
        }
    }

    pub fn open_dir(&mut self, path: &str) -> Result<Inode, Error<()>> {
        let (name, rest_opt) = split_path(path);
        let mut e = self.find_entry(name, Some(true)).unwrap();
        match rest_opt {
            Some(rest) => e.open_dir(rest),
            None => Ok(Inode::Dir(e)),
        }
    }


    pub fn to_file(&self) -> FileEntry {
        FileEntry::from(self.dir_entry, self.entry_pos, self.file_name())
    }

    pub fn remove(&mut self, path: &str) -> Result<(), Error<()>> {
        let (name, rest_opt) = split_path(path);
        if let Some(rest) = rest_opt {
            let mut e = self.find_entry(name, Some(true))?;
            return e.remove(rest);
        }
        let mut e = self.find_entry(name, None)?;
        if e.is_dir() && !e.is_empty() {
            return Err(Error::DirectoryIsNotEmpty);
        }
        if let Some(_n) = e.dir_entry.first_cluster() {}
        let mut stream = self.disk.inner.borrow_mut();
        stream.seek(SeekFrom::Start(e.offset_range.0))?;
        let num = ((e.offset_range.1 - e.offset_range.0) / u64::from(DIR_ENTRY_SIZE)) as usize;
        for _ in 0..num {
            let mut data = DirEntryData::deserialize(&mut *stream)?;
            data.set_deleted();
            stream.seek(SeekFrom::Current(-i64::from(DIR_ENTRY_SIZE)))?;
            data.serialize(&mut *stream)?;
        }
        Ok(())
    }

    pub fn short_file_name_as_bytes(&self) -> &[u8] {
        self.short_name.as_bytes()
    }

    pub fn is_empty(&mut self) -> bool {
        self.seek(SeekFrom::Current(0)).unwrap();
        for r in self {
            let name = r.short_file_name_as_bytes();
            if name != b"." && name != b".." {
                return false;
            }
        }
        true
    }
}

impl IoBase for DirEntry {
    type Error = ();
}

impl Seek for DirEntry {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let new_offset = match pos {
            SeekFrom::Start(x) => {
                let start_pos = cluster_to_offset(self.dir_entry.first_cluster().unwrap()) + x;
                self.offset = start_pos;
                start_pos
            }
            SeekFrom::End(_) => 0,
            SeekFrom::Current(x) => {
                self.offset += x as u64;
                self.offset
            }
        };
        self.disk
            .inner
            .borrow_mut()
            .seek(SeekFrom::Start(new_offset))
            .unwrap();
        Ok(new_offset)
    }
}

impl Read for DirEntry {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut disk = self.disk.inner.borrow_mut();
        self.offset += buf.len() as u64;

        disk.read(buf)
    }
}

impl Iterator for DirEntry {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.read_next_entry();
        match r {
            Ok(Some(e)) => Some(e),
            Ok(None) => None,
            Err(_) => None,
        }
    }
}

fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

pub enum DirEntryOrShortName {
    DirEntry(DirEntry),
    ShortName([u8; SFN_SIZE]),
}

#[derive(Clone, Debug)]
pub struct DirEntryEditor {
    pub data: DirFileEntry,
    pub pos: u64,
    pub dirty: bool,
}

impl DirEntryEditor {
    pub fn new(data: DirFileEntry, pos: u64) -> Self {
        Self {
            data,
            pos,
            dirty: false,
        }
    }

    pub fn inner(&mut self) -> &mut DirFileEntry {
        &mut self.data
    }

    pub fn set_first_cluster(&mut self, first_cluster: Option<u32>) {
        if first_cluster != self.data.first_cluster() {
            self.data.set_first_cluster(first_cluster);
            self.dirty = true;
        }
    }

    pub fn set_size(&mut self, size: u32) {
        if self.data.size != size {
            self.data.set_size(size)
        }
    }

    pub fn flush(&self) {
        let mut disk = BlockCacheManager::new();

        disk.seek(io::SeekFrom::Start(self.pos)).unwrap();

        self.data.serialize(&mut disk).unwrap();
    }
}
