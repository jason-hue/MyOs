use core::{char::ToUppercase, cmp, iter, num, str};

use alloc::{slice, string::String, vec::Vec};
use log::warn;

use super::{
    dir_entry::DirLfnEntry,
    io::{Error, IoError},
};
pub const LFN_PADDING: u16 = 0xFFFF;
pub const LFN_PART_LEN: usize = 13;
pub const DIR_ENTRY_SIZE: u32 = 32;
pub const DIR_ENTRY_DELETED_FLAG: u8 = 0xE5;
pub const LFN_ENTRY_LAST_FLAG: u8 = 0x40;
pub const MAX_LONG_NAME_LEN: usize = 255;
const MAX_LONG_DIR_ENTRIES: usize = (MAX_LONG_NAME_LEN + LFN_PART_LEN - 1) / LFN_PART_LEN;
pub const SFN_SIZE: usize = 11;
pub const SFN_PADDING: u8 = b' ';
pub const DIR_ENTRY_REALLY_E5_FLAG: u8 = 0x05;
pub struct ShortName {
    name: [u8; 12],
    len: u8,
}

impl ShortName {
    pub fn new(raw_name: &[u8; SFN_SIZE]) -> Self {
        // get name components length by looking for space character
        let name_len = raw_name[0..8]
            .iter()
            .rposition(|x| *x != SFN_PADDING)
            .map_or(0, |p| p + 1);
        let ext_len = raw_name[8..11]
            .iter()
            .rposition(|x| *x != SFN_PADDING)
            .map_or(0, |p| p + 1);
        let mut name = [SFN_PADDING; 12];
        name[..name_len].copy_from_slice(&raw_name[..name_len]);
        let total_len = if ext_len > 0 {
            name[name_len] = b'.';
            name[name_len + 1..name_len + 1 + ext_len].copy_from_slice(&raw_name[8..8 + ext_len]);
            // Return total name length
            name_len + 1 + ext_len
        } else {
            // No extension - return length of name part
            name_len
        };
        // FAT encodes character 0xE5 as 0x05 because 0xE5 marks deleted files
        if name[0] == DIR_ENTRY_REALLY_E5_FLAG {
            name[0] = 0xE5;
        }
        // Short names in FAT filesystem are encoded in OEM code-page
        Self {
            name,
            len: total_len as u8,
        }
    }

    pub fn to_string(&self) -> String {
        self.as_bytes()
            .iter()
            .copied()
            .map(|oem_char| {
                if oem_char <= 0x7F {
                    char::from(oem_char)
                } else {
                    '\u{FFFD}'
                }
            })
            .collect()
    }

    pub fn eq_ignore_case(&self, name: &str) -> bool {
        // Convert name to UTF-8 character iterator
        let byte_iter = self.as_bytes().iter().copied();
        let char_iter = byte_iter.map(|oem_char| {
            if oem_char <= 0x7F {
                char::from(oem_char)
            } else {
                '\u{FFFD}'
            }
        });
        // Compare interators ignoring case
        let uppercase_char_iter = char_iter.flat_map(char_to_uppercase);
        uppercase_char_iter.eq(name.chars().flat_map(char_to_uppercase))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.name[..usize::from(self.len)]
    }
}
pub fn char_to_uppercase(c: char) -> ToUppercase {
    c.to_uppercase()
}
#[derive(Clone)]
pub struct LfnBuffer {
    ucs2_units: Vec<u16>,
}

impl LfnBuffer {
    pub fn new() -> Self {
        Self {
            ucs2_units: Vec::<u16>::new(),
        }
    }

    pub fn from_ucs2_units<I: Iterator<Item = u16>>(usc2_units: I) -> Self {
        Self {
            ucs2_units: usc2_units.collect(),
        }
    }

    fn clear(&mut self) {
        self.ucs2_units.clear();
    }

    pub fn len(&self) -> usize {
        self.ucs2_units.len()
    }

    fn set_len(&mut self, len: usize) {
        self.ucs2_units.resize(len, 0_u16);
    }

    pub fn as_ucs2_units(&self) -> &[u16] {
        &self.ucs2_units
    }
}

pub struct LongNameBuilder {
    buf: LfnBuffer,
    chksum: u8,
    index: u8,
}

impl LongNameBuilder {
    pub fn new() -> Self {
        Self {
            buf: LfnBuffer::new(),
            chksum: 0,
            index: 0,
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.index = 0;
    }

    pub fn into_buf(mut self) -> LfnBuffer {
        // Check if last processed entry had index 1
        if self.index == 1 {
            self.truncate();
        } else if !self.is_empty() {
            warn!("unfinished LFN sequence {}", self.index);
            self.clear();
        }
        self.buf
    }

    pub fn truncate(&mut self) {
        // Truncate 0 and 0xFFFF characters from LFN buffer
        let ucs2_units = &self.buf.ucs2_units;
        let new_len = ucs2_units
            .iter()
            .rposition(|c| *c != 0xFFFF && *c != 0)
            .map_or(0, |n| n + 1);
        self.buf.set_len(new_len);
    }

    pub fn is_empty(&self) -> bool {
        // Check if any LFN entry has been processed
        // Note: index 0 is not a valid index in LFN and can be seen only after struct initialization
        self.index == 0
    }

    pub fn process(&mut self, data: &DirLfnEntry) {
        let is_last = (data.order() & LFN_ENTRY_LAST_FLAG) != 0;
        let index = data.order() & 0x1F;
        if index == 0 || usize::from(index) > MAX_LONG_DIR_ENTRIES {
            // Corrupted entry
            warn!("currupted lfn entry! {:x}", data.order());
            self.clear();
            return;
        }
        if is_last {
            // last entry is actually first entry in stream
            self.index = index;
            self.chksum = data.checksum();
            self.buf.set_len(usize::from(index) * LFN_PART_LEN);
        } else if self.index == 0 || index != self.index - 1 || data.checksum() != self.chksum {
            // Corrupted entry
            warn!(
                "currupted lfn entry! {:x} {:x} {:x} {:x}",
                data.order(),
                self.index,
                data.checksum(),
                self.chksum
            );
            self.clear();
            return;
        } else {
            // Decrement LFN index only for non-last entries
            self.index -= 1;
        }
        let pos = LFN_PART_LEN * usize::from(index - 1);
        // copy name parts into LFN buffer
        data.copy_name_to_slice(&mut self.buf.ucs2_units[pos..pos + 13]);
    }

    pub fn validate_chksum(&mut self, short_name: &[u8; SFN_SIZE]) {
        if self.is_empty() {
            // Nothing to validate - no LFN entries has been processed
            return;
        }
        let chksum = lfn_checksum(short_name);
        if chksum != self.chksum {
            warn!(
                "checksum mismatch {:x} {:x} {:?}",
                chksum, self.chksum, short_name
            );
            self.clear();
        }
    }
}

pub fn lfn_checksum(short_name: &[u8; SFN_SIZE]) -> u8 {
    let mut chksum = num::Wrapping(0_u8);
    for b in short_name {
        chksum = (chksum << 7) + (chksum >> 1) + num::Wrapping(*b);
    }
    chksum.0
}

#[derive(Default, Debug, Clone)]
pub struct ShortNameGenerator {
    chksum: u16,
    long_prefix_bitmap: u16,
    prefix_chksum_bitmap: u16,
    name_fits: bool,
    lossy_conv: bool,
    exact_match: bool,
    basename_len: usize,
    short_name: [u8; SFN_SIZE],
}

impl ShortNameGenerator {
    pub fn new(name: &str) -> Self {
        // padded by ' '
        let mut short_name = [SFN_PADDING; SFN_SIZE];
        // find extension after last dot
        // Note: short file name cannot start with the extension
        let dot_index_opt = name[1..].rfind('.').map(|index| index + 1);
        // copy basename (part of filename before a dot)
        let basename_src = dot_index_opt.map_or(name, |dot_index| &name[..dot_index]);
        let (basename_len, basename_fits, basename_lossy) =
            Self::copy_short_name_part(&mut short_name[0..8], basename_src);
        // copy file extension if exists
        let (name_fits, lossy_conv) =
            dot_index_opt.map_or((basename_fits, basename_lossy), |dot_index| {
                let (_, ext_fits, ext_lossy) =
                    Self::copy_short_name_part(&mut short_name[8..11], &name[dot_index + 1..]);
                (basename_fits && ext_fits, basename_lossy || ext_lossy)
            });
        let chksum = Self::checksum(name);
        Self {
            chksum,
            name_fits,
            lossy_conv,
            basename_len,
            short_name,
            ..Self::default()
        }
    }

    pub fn generate_dot() -> [u8; SFN_SIZE] {
        let mut short_name = [SFN_PADDING; SFN_SIZE];
        short_name[0] = b'.';
        short_name
    }

    pub fn generate_dotdot() -> [u8; SFN_SIZE] {
        let mut short_name = [SFN_PADDING; SFN_SIZE];
        short_name[0] = b'.';
        short_name[1] = b'.';
        short_name
    }

    pub fn copy_short_name_part(dst: &mut [u8], src: &str) -> (usize, bool, bool) {
        let mut dst_pos = 0;
        let mut lossy_conv = false;
        for c in src.chars() {
            if dst_pos == dst.len() {
                // result buffer is full
                return (dst_pos, false, lossy_conv);
            }
            // Make sure character is allowed in 8.3 name
            #[rustfmt::skip]
                let fixed_c = match c {
                // strip spaces and dots
                ' ' | '.' => {
                    lossy_conv = true;
                    continue;
                },
                // copy allowed characters
                'A'..='Z' | 'a'..='z' | '0'..='9'
                | '!' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '-' | '@' | '^' | '_' | '`' | '{' | '}' | '~' => c,
                // replace disallowed characters by underscore
                _ => '_',
            };
            // Update 'lossy conversion' flag
            lossy_conv = lossy_conv || (fixed_c != c);
            // short name is always uppercase
            let upper = fixed_c.to_ascii_uppercase();
            dst[dst_pos] = upper as u8; // SAFE: upper is in range 0x20-0x7F
            dst_pos += 1;
        }
        (dst_pos, true, lossy_conv)
    }

    pub fn add_existing(&mut self, short_name: &[u8; SFN_SIZE]) {
        // check for exact match collision
        if short_name == &self.short_name {
            self.exact_match = true;
        }
        // check for long prefix form collision (TEXTFI~1.TXT)
        self.check_for_long_prefix_collision(short_name);

        // check for short prefix + checksum form collision (TE021F~1.TXT)
        self.check_for_short_prefix_collision(short_name);
    }

    pub fn check_for_long_prefix_collision(&mut self, short_name: &[u8; SFN_SIZE]) {
        // check for long prefix form collision (TEXTFI~1.TXT)
        let long_prefix_len = cmp::min(self.basename_len, 6);
        if short_name[long_prefix_len] != b'~' {
            return;
        }
        if let Some(num_suffix) = char::from(short_name[long_prefix_len + 1]).to_digit(10) {
            let long_prefix_matches =
                short_name[..long_prefix_len] == self.short_name[..long_prefix_len];
            let ext_matches = short_name[8..] == self.short_name[8..];
            if long_prefix_matches && ext_matches {
                self.long_prefix_bitmap |= 1 << num_suffix;
            }
        }
    }

    pub fn check_for_short_prefix_collision(&mut self, short_name: &[u8; SFN_SIZE]) {
        // check for short prefix + checksum form collision (TE021F~1.TXT)
        let short_prefix_len = cmp::min(self.basename_len, 2);
        if short_name[short_prefix_len + 4] != b'~' {
            return;
        }
        if let Some(num_suffix) = char::from(short_name[short_prefix_len + 4 + 1]).to_digit(10) {
            let short_prefix_matches =
                short_name[..short_prefix_len] == self.short_name[..short_prefix_len];
            let ext_matches = short_name[8..] == self.short_name[8..];
            if short_prefix_matches && ext_matches {
                let chksum_res =
                    str::from_utf8(&short_name[short_prefix_len..short_prefix_len + 4])
                        .map(|s| u16::from_str_radix(s, 16));
                if chksum_res == Ok(Ok(self.chksum)) {
                    self.prefix_chksum_bitmap |= 1 << num_suffix;
                }
            }
        }
    }

    pub fn checksum(name: &str) -> u16 {
        // BSD checksum algorithm
        let mut chksum = num::Wrapping(0_u16);
        for c in name.chars() {
            chksum = (chksum >> 1) + (chksum << 15) + num::Wrapping(c as u16);
        }
        chksum.0
    }

    pub fn generate(&self) -> Result<[u8; SFN_SIZE], Error<()>> {
        if !self.lossy_conv && self.name_fits && !self.exact_match {
            // If there was no lossy conversion and name fits into
            // 8.3 convention and there is no collision return it as is
            return Ok(self.short_name);
        }
        // Try using long 6-characters prefix
        for i in 1..5 {
            if self.long_prefix_bitmap & (1 << i) == 0 {
                return Ok(self.build_prefixed_name(i, false));
            }
        }
        // Try prefix with checksum
        for i in 1..10 {
            if self.prefix_chksum_bitmap & (1 << i) == 0 {
                return Ok(self.build_prefixed_name(i, true));
            }
        }
        // Too many collisions - fail
        Err(Error::AlreadyExists)
    }

    pub fn next_iteration(&mut self) {
        // Try different checksum in next iteration
        self.chksum = (num::Wrapping(self.chksum) + num::Wrapping(1)).0;
        // Zero bitmaps
        self.long_prefix_bitmap = 0;
        self.prefix_chksum_bitmap = 0;
    }

    fn build_prefixed_name(&self, num: u32, with_chksum: bool) -> [u8; SFN_SIZE] {
        let mut buf = [SFN_PADDING; SFN_SIZE];
        let prefix_len = if with_chksum {
            let prefix_len = cmp::min(self.basename_len, 2);
            buf[..prefix_len].copy_from_slice(&self.short_name[..prefix_len]);
            buf[prefix_len..prefix_len + 4].copy_from_slice(&Self::u16_to_hex(self.chksum));
            prefix_len + 4
        } else {
            let prefix_len = cmp::min(self.basename_len, 6);
            buf[..prefix_len].copy_from_slice(&self.short_name[..prefix_len]);
            prefix_len
        };
        buf[prefix_len] = b'~';
        buf[prefix_len + 1] = char::from_digit(num, 10).unwrap() as u8; // SAFE: num is in range [1, 9]
        buf[8..].copy_from_slice(&self.short_name[8..]);
        buf
    }

    fn u16_to_hex(x: u16) -> [u8; 4] {
        // Unwrapping below is safe because each line takes 4 bits of `x` and shifts them to the right so they form
        // a number in range [0, 15]
        let x_u32 = u32::from(x);
        let mut hex_bytes = [
            char::from_digit((x_u32 >> 12) & 0xF, 16).unwrap() as u8,
            char::from_digit((x_u32 >> 8) & 0xF, 16).unwrap() as u8,
            char::from_digit((x_u32 >> 4) & 0xF, 16).unwrap() as u8,
            char::from_digit(x_u32 & 0xF, 16).unwrap() as u8,
        ];
        hex_bytes.make_ascii_uppercase();
        hex_bytes
    }
}

pub struct LfnEntriesGenerator<'a> {
    name_parts_iter: iter::Rev<slice::Chunks<'a, u16>>,
    checksum: u8,
    index: usize,
    num: usize,
    ended: bool,
}

impl<'a> LfnEntriesGenerator<'a> {
    pub fn new(name_utf16: &'a [u16], checksum: u8) -> Self {
        let num_entries = (name_utf16.len() + LFN_PART_LEN - 1) / LFN_PART_LEN;
        // create generator using reverse iterator over chunks - first chunk can be shorter
        LfnEntriesGenerator {
            checksum,
            name_parts_iter: name_utf16.chunks(LFN_PART_LEN).rev(),
            index: 0,
            num: num_entries,
            ended: false,
        }
    }
}

impl Iterator for LfnEntriesGenerator<'_> {
    type Item = DirLfnEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        // get next part from reverse iterator
        if let Some(name_part) = self.name_parts_iter.next() {
            let lfn_index = self.num - self.index;
            let mut order = lfn_index as u8;
            if self.index == 0 {
                // this is last name part (written as first)
                order |= LFN_ENTRY_LAST_FLAG;
            }
            debug_assert!(order > 0);
            let mut lfn_part = [LFN_PADDING; LFN_PART_LEN];
            lfn_part[..name_part.len()].copy_from_slice(name_part);
            if name_part.len() < LFN_PART_LEN {
                // name is only zero-terminated if its length is not multiplicity of LFN_PART_LEN
                lfn_part[name_part.len()] = 0;
            }
            // create and return new LFN entry
            let mut lfn_entry = DirLfnEntry::new(order, self.checksum);
            lfn_entry.copy_name_from_slice(&lfn_part);
            self.index += 1;
            Some(lfn_entry)
        } else {
            // end of name
            self.ended = true;
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.name_parts_iter.size_hint()
    }
}

impl ExactSizeIterator for LfnEntriesGenerator<'_> {}

#[rustfmt::skip]
pub fn validate_long_name<E: IoError>(name: &str) -> Result<(), Error<E>> {
    if name.is_empty() {
        return Err(Error::InvalidFileNameLength);
    }
    if name.len() > MAX_LONG_NAME_LEN {
        return Err(Error::InvalidFileNameLength);
    }
    for c in name.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9'
            | '\u{80}'..='\u{FFFF}'
            | '$' | '%' | '\'' | '-' | '_' | '@' | '~' | '`' | '!' | '(' | ')' | '{' | '}' | '.' | ' ' | '+' | ','
            | ';' | '=' | '[' | ']' | '^' | '#' | '&' => {},
            _ => return Err(Error::UnsupportedFileNameCharacter),
        }
    }
    Ok(())
}
