use log::{error, info};

use super::io::{Read, ReadLeExt};

#[derive(Default, Debug, Clone)]
pub struct BiosParameterBlock {
    pub bytes_per_sector: u16,      //每个扇区的字节数，通常为512
    pub sectors_per_cluster: u8,    //每个簇包含的扇区数
    pub reserved_sectors: u16,      
    pub fats: u8,                   //fat表数量，为2
    pub root_entries: u16,
    pub total_sectors_16: u16,
    #[allow(unused)]
    pub media: u8,
    pub sectors_per_fat_16: u16,
    #[allow(unused)]
    pub sectors_per_track: u16,
    #[allow(unused)]
    pub heads: u16,
    #[allow(unused)]
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,

    // Extended BIOS Parameter Block
    pub sectors_per_fat_32: u32,    //每个fat表所占的扇区数
    pub extended_flags: u16,
    pub fs_version: u16,
    pub root_dir_first_cluster: u32,//根目录的第一个簇，通常用于初始化
    pub fs_info_sector: u16,
    pub backup_boot_sector: u16,
    pub reserved_0: [u8; 12],
    pub drive_num: u8,
    pub reserved_1: u8,
    pub ext_sig: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type_label: [u8; 8],
}

pub struct BootSector {
    bootjmp: [u8; 3],
    oem_name: [u8; 8],
    pub bpb: BiosParameterBlock,
    boot_code: [u8; 448], //引导代码
    boot_sig: [u8; 2], //引导扇区标志，标识有效性
}

impl Default for BootSector {
    fn default() -> Self {
        Self {
            bootjmp: Default::default(),
            oem_name: Default::default(),
            bpb: BiosParameterBlock::default(),
            boot_code: [0; 448],
            boot_sig: Default::default(),
        }
    }
}

impl BiosParameterBlock {
        // 顺序读取设备R的第一个扇区的内容来初始化BPB
    fn deserialize<R: Read>(rdr: &mut R) -> Result<Self, R::Error> {
        let mut bpb = Self {
            bytes_per_sector: rdr.read_u16_le()?,
            sectors_per_cluster: rdr.read_u8()?,
            reserved_sectors: rdr.read_u16_le()?,
            fats: rdr.read_u8()?,
            root_entries: rdr.read_u16_le()?,
            total_sectors_16: rdr.read_u16_le()?,
            media: rdr.read_u8()?,
            sectors_per_fat_16: rdr.read_u16_le()?,
            sectors_per_track: rdr.read_u16_le()?,
            heads: rdr.read_u16_le()?,
            hidden_sectors: rdr.read_u32_le()?,
            total_sectors_32: rdr.read_u32_le()?,
            ..Self::default()
        };

        if bpb.is_fat32() {
            bpb.sectors_per_fat_32 = rdr.read_u32_le()?;
            bpb.extended_flags = rdr.read_u16_le()?;
            bpb.fs_version = rdr.read_u16_le()?;
            bpb.root_dir_first_cluster = rdr.read_u32_le()?;
            bpb.fs_info_sector = rdr.read_u16_le()?;
            bpb.backup_boot_sector = rdr.read_u16_le()?;
            rdr.read_exact(&mut bpb.reserved_0)?;
        }

        bpb.drive_num = rdr.read_u8()?;
        bpb.reserved_1 = rdr.read_u8()?;
        bpb.ext_sig = rdr.read_u8()?; // 0x29
        bpb.volume_id = rdr.read_u32_le()?;
        rdr.read_exact(&mut bpb.volume_label)?;
        rdr.read_exact(&mut bpb.fs_type_label)?;

        // when the extended boot signature is anything other than 0x29, the fields are invalid
        if bpb.ext_sig != 0x29 {
            // fields after ext_sig are not used - clean them
            bpb.volume_id = 0;
            bpb.volume_label = [0; 11];
            bpb.fs_type_label = [0; 8];
        }

        Ok(bpb)
    }
    pub fn is_fat32(&self) -> bool {
        self.sectors_per_fat_16 == 0
    }
    pub fn root_dir_sectors(&self) -> u32 {
        let root_dir_bytes = u32::from(self.root_entries) * 32;
        (root_dir_bytes + u32::from(self.bytes_per_sector) - 1) / u32::from(self.bytes_per_sector)
    }

    pub fn sectors_per_fat(&self) -> u32 {
        if self.is_fat32() {
            self.sectors_per_fat_32
        } else {
            u32::from(self.sectors_per_fat_16)
        }
    }

    pub fn sectors_per_all_fats(&self) -> u32 {
        u32::from(self.fats) * self.sectors_per_fat()
    }

    pub fn reserved_sectors(&self) -> u32 {
        u32::from(self.reserved_sectors)
    }

    pub fn first_data_sector(&self) -> u32 {
        let root_dir_sectors = self.root_dir_sectors();
        let fat_sectors = self.sectors_per_all_fats();
        self.reserved_sectors() + fat_sectors + root_dir_sectors
    }

    pub fn total_sectors(&self) -> u32 {
        if self.total_sectors_16 == 0 {
            self.total_sectors_32
        } else {
            u32::from(self.total_sectors_16)
        }
    }
    
    pub fn total_clusters(&self) -> u32 {
        let total_sectors = self.total_sectors();
        let first_data_sector = self.first_data_sector();
        let data_sectors = total_sectors - first_data_sector;
        data_sectors / u32::from(self.sectors_per_cluster)
    }
    pub fn fs_info_sector(&self) -> u32 {
        u32::from(self.fs_info_sector)
    }
    pub fn bytes_from_sectors(&self, sectors: u32) -> u64 {
        u64::from(sectors) * u64::from(self.bytes_per_sector)
    }
    pub fn sectors_from_clusters(&self, clusters: u32) -> u32 {
        clusters * u32::from(self.sectors_per_cluster)
    }
    pub fn cluster_size(&self) -> u32 {
        u32::from(self.sectors_per_cluster) * u32::from(self.bytes_per_sector)
    }
    pub fn mirroring_enabled(&self) -> bool {
        self.extended_flags & 0x80 == 0
    }
    pub fn active_fat(&self) -> u16 {
        if self.mirroring_enabled() {
            0
        } else {
            self.extended_flags & 0x0F
        }
    }
}

impl BootSector {
    pub fn deserialize<R: Read>(rdr: &mut R) -> Result<Self, R::Error> {
        let mut boot = Self::default();
        rdr.read_exact(&mut boot.bootjmp)?;
        rdr.read_exact(&mut boot.oem_name)?;
        boot.bpb = BiosParameterBlock::deserialize(rdr)?;

        if boot.bpb.is_fat32() {
            rdr.read_exact(&mut boot.boot_code[0..420])?;
        } else {
            rdr.read_exact(&mut boot.boot_code[0..448])?;
        }
        rdr.read_exact(&mut boot.boot_sig)?;
        Ok(boot)
    }
}
