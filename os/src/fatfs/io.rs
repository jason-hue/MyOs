use alloc::vec::Vec;
use log::debug;

#[derive(Debug)]
#[non_exhaustive]
/// Provides IO error as an associated type.
pub enum Error<T> {
    Io(T),
    UnexpectedEof,
    WriteZero,
    InvalidInput,
    NotFound,
    AlreadyExists,
    DirectoryIsNotEmpty,
    CorruptedFileSystem,
    NotEnoughSpace,
    InvalidFileNameLength,
    UnsupportedFileNameCharacter,
}

impl<T: IoError> From<T> for Error<T> {
    fn from(error: T) -> Self {
        Error::Io(error)
    }
}

impl<T: core::fmt::Display> core::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Io(io_error) => write!(f, "IO error: {}", io_error),
            Error::UnexpectedEof => write!(f, "Unexpected end of file"),
            Error::NotEnoughSpace => write!(f, "Not enough space"),
            Error::WriteZero => write!(f, "Write zero"),
            Error::InvalidInput => write!(f, "Invalid input"),
            Error::InvalidFileNameLength => write!(f, "Invalid file name length"),
            Error::UnsupportedFileNameCharacter => write!(f, "Unsupported file name character"),
            Error::DirectoryIsNotEmpty => write!(f, "Directory is not empty"),
            Error::NotFound => write!(f, "No such file or directory"),
            Error::AlreadyExists => write!(f, "File or directory already exists"),
            Error::CorruptedFileSystem => write!(f, "Corrupted file system"),
        }
    }
}

pub trait IoError: core::fmt::Debug {
    fn is_interrupted(&self) -> bool;
    fn new_unexpected_eof_error() -> Self;
    fn new_write_zero_error() -> Self;
}

impl<T: core::fmt::Debug + IoError> IoError for Error<T> {
    fn is_interrupted(&self) -> bool {
        match self {
            Error::<T>::Io(io_error) => io_error.is_interrupted(),
            _ => false,
        }
    }

    fn new_unexpected_eof_error() -> Self {
        Error::<T>::UnexpectedEof
    }

    fn new_write_zero_error() -> Self {
        Error::<T>::WriteZero
    }
}

impl IoError for () {
    fn is_interrupted(&self) -> bool {
        false
    }

    fn new_unexpected_eof_error() -> Self {}

    fn new_write_zero_error() -> Self {}
}

pub trait IoBase {
    type Error: IoError;
}

pub trait Read: IoBase {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Self::Error> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    
                }
                Err(ref e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
        if buf.is_empty() {
            Ok(())
        } else {
            debug!("failed to fill whole buffer in read_exact");
            Err(Self::Error::new_unexpected_eof_error())
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Self::Error> {
        let mut size = 0;
        let mut cache = [0; 512];
        loop {
            match self.read(&mut cache) {
                Ok(len) => {
                    for i in 0..len {
                        buf.push(cache[i])
                    }
                    size += len;
                    if len < 512 {
                        break;
                    }
                }
                Err(_) => todo! {},
            }
        }
        Ok(size)
    }
}
pub trait Write: IoBase {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Self::Error> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    debug!("failed to write whole buffer in write_all");
                    return Err(Self::Error::new_write_zero_error());
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error>;
}
#[derive(Debug)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub trait Seek: IoBase {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error>;
}

pub trait ReadLeExt {
    type Error;
    fn read_u8(&mut self) -> Result<u8, Self::Error>;
    fn read_u16_le(&mut self) -> Result<u16, Self::Error>;
    fn read_u32_le(&mut self) -> Result<u32, Self::Error>;
}

impl<T: Read> ReadLeExt for T {
    type Error = <Self as IoBase>::Error;

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        let mut buf = [0_u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u16_le(&mut self) -> Result<u16, Self::Error> {
        let mut buf = [0_u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_u32_le(&mut self) -> Result<u32, Self::Error> {
        let mut buf = [0_u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

pub trait WriteLeExt {
    type Error;
    fn write_u8(&mut self, n: u8) -> Result<(), Self::Error>;
    fn write_u16_le(&mut self, n: u16) -> Result<(), Self::Error>;
    fn write_u32_le(&mut self, n: u32) -> Result<(), Self::Error>;
}

impl<T: Write> WriteLeExt for T {
    type Error = <Self as IoBase>::Error;

    fn write_u8(&mut self, n: u8) -> Result<(), Self::Error> {
        self.write_all(&[n])
    }

    fn write_u16_le(&mut self, n: u16) -> Result<(), Self::Error> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_u32_le(&mut self, n: u32) -> Result<(), Self::Error> {
        self.write_all(&n.to_le_bytes())
    }
}

pub trait ReadWriteSeek: Read + Write + Seek {}
impl<T: Read + Write + Seek> ReadWriteSeek for T {}
