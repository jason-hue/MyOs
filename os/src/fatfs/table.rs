use core::marker::PhantomData;

use log::{trace, warn};

use super::io::{self, Error, Read, ReadLeExt, Seek, Write, WriteLeExt};
pub const RESERVED_FAT_ENTRIES: u32 = 2;
struct Fat<S> {
    phantom: PhantomData<S>,
}
type Fat32 = Fat<u32>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum FatValue {
    Free,
    Data(u32),
    Bad,
    EndOfChain,
}

trait FatTrait {
    fn get_raw<S>(fat: &mut S, cluster: u32) -> Result<u32, Error<()>>
        where
            S: Read + Seek;

    fn get<S>(fat: &mut S, cluster: u32) -> Result<FatValue, Error<()>>
        where
            S: Read + Seek;

    fn set_raw<S>(fat: &mut S, cluster: u32, raw_value: u32) -> Result<(), Error<()>>
        where
            S: Read + Write + Seek;

    fn set<S>(fat: &mut S, cluster: u32, value: FatValue) -> Result<(), Error<()>>
        where
            S: Read + Write + Seek;

    fn find_free<S>(fat: &mut S, start_cluster: u32, end_cluster: u32) -> Result<u32, Error<()>>
        where
            S: Read + Seek;

    fn count_free<S>(fat: &mut S, end_cluster: u32) -> Result<u32, Error<()>>
        where
            S: Read + Seek;
}

impl FatTrait for Fat32 {
    fn get_raw<S>(fat: &mut S, cluster: u32) -> Result<u32, Error<()>>
        where
            S: Read + Seek,
    {
        fat.seek(io::SeekFrom::Start(u64::from(cluster * 4)))
            .unwrap();
        Ok(fat.read_u32_le().unwrap())
    }

    fn get<S>(fat: &mut S, cluster: u32) -> Result<FatValue, Error<()>>
        where
            S: Read + Seek,
    {
        let val = Self::get_raw(fat, cluster)? & 0x0FFF_FFFF;
        Ok(match val {
            0 if (0x0FFF_FFF7..=0x0FFF_FFFF).contains(&cluster) => {
                let tmp = if cluster == 0x0FFF_FFF7 {
                    "BAD_CLUSTER"
                } else {
                    "end-of-chain"
                };
                warn!(
                        "cluster number {} is a special value in FAT to indicate {}; it should never be seen as free",
                        cluster, tmp
                    );
                FatValue::Bad // avoid accidental use or allocation into a FAT chain
            }
            0 => FatValue::Free,
            0x0FFF_FFF7 => FatValue::Bad,
            0x0FFF_FFF8..=0x0FFF_FFFF => FatValue::EndOfChain,
            n if (0x0FFF_FFF7..=0x0FFF_FFFF).contains(&cluster) => {
                let tmp = if cluster == 0x0FFF_FFF7 {
                    "BAD_CLUSTER"
                } else {
                    "end-of-chain"
                };
                warn!("cluster number {} is a special value in FAT to indicate {}; hiding potential FAT chain value {} and instead reporting as a bad sector", cluster, tmp, n);
                FatValue::Bad // avoid accidental use or allocation into a FAT chain
            }
            n => FatValue::Data(n),
        })
    }

    fn set_raw<S>(fat: &mut S, cluster: u32, raw_value: u32) -> Result<(), Error<()>>
        where
            S: Read + Write + Seek,
    {
        fat.seek(io::SeekFrom::Start(u64::from(cluster * 4)))
            .unwrap();
        fat.write_u32_le(raw_value).unwrap();
        Ok(())
    }

    fn set<S>(fat: &mut S, cluster: u32, value: FatValue) -> Result<(), Error<()>>
        where
            S: Read + Write + Seek,
    {
        let old_reserved_bits = Self::get_raw(fat, cluster)? & 0xF000_0000;

        if value == FatValue::Free && cluster >= 0x0FFF_FFF7 && cluster <= 0x0FFF_FFFF {
            // NOTE: it is technically allowed for them to store FAT chain loops,
            //       or even have them all store value '4' as their next cluster.
            //       Some believe only FatValue::Bad should be allowed for this edge case.
            let tmp = if cluster == 0x0FFF_FFF7 {
                "BAD_CLUSTER"
            } else {
                "end-of-chain"
            };
            panic!(
                "cluster number {} is a special value in FAT to indicate {}; it should never be set as free",
                cluster, tmp
            );
        };
        let raw_val = match value {
            FatValue::Free => 0,
            FatValue::Bad => 0x0FFF_FFF7,
            FatValue::EndOfChain => 0x0FFF_FFFF,
            FatValue::Data(n) => n,
        };
        let raw_val = raw_val | old_reserved_bits; // must preserve original reserved values
        Self::set_raw(fat, cluster, raw_val)
    }

    fn find_free<S>(fat: &mut S, start_cluster: u32, end_cluster: u32) -> Result<u32, Error<()>>
        where
            S: Read + Seek,
    {
        let mut cluster = start_cluster;

        fat.seek(io::SeekFrom::Start(u64::from(cluster * 4)))
            .unwrap();
        while cluster < end_cluster {
            let val = fat.read_u32_le().unwrap() & 0x0FFF_FFFF;
            if val == 0 {
                return Ok(cluster);
            }
            cluster += 1;
        }
        Err(Error::NotEnoughSpace)
    }

    fn count_free<S>(fat: &mut S, end_cluster: u32) -> Result<u32, Error<()>>
        where
            S: Read + Seek,
    {
        let mut count = 0;
        let mut cluster = RESERVED_FAT_ENTRIES;
        fat.seek(io::SeekFrom::Start(u64::from(cluster * 4)))
            .unwrap();
        while cluster < end_cluster {
            let val = fat.read_u32_le().unwrap() & 0x0FFF_FFFF;
            if val == 0 {
                count += 1;
            }
            cluster += 1;
        }
        Ok(count)
    }
}

pub fn table_alloc_cluster<S>(
    fat: &mut S,
    prev_cluster: Option<u32>,
    hint: Option<u32>,
    total_clusters: u32,
) -> Result<u32, Error<()>>
    where
        S: Read + Write + Seek,
{
    let end_cluster = total_clusters + RESERVED_FAT_ENTRIES;
    let start_cluster = match hint {
        Some(n) if n < end_cluster => n,
        _ => RESERVED_FAT_ENTRIES,
    };
    let new_cluster = match Fat32::find_free(fat, start_cluster, end_cluster) {
        Ok(n) => n,
        Err(_) if start_cluster > RESERVED_FAT_ENTRIES => {
            Fat32::find_free(fat, RESERVED_FAT_ENTRIES, end_cluster).unwrap()
        }
        Err(e) => return Err(e),
    };
    Fat32::set(fat, new_cluster, FatValue::EndOfChain).unwrap();
    if let Some(n) = prev_cluster {
        Fat32::set(fat, n, FatValue::Data(new_cluster)).unwrap();
    }
    trace!("allocated cluster {}", new_cluster);
    Ok(new_cluster)
}
