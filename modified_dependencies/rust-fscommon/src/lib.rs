#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate core2;

#[macro_use]
extern crate log;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
use std::io;

use core2::io;

mod buf_stream;
mod stream_slice;

pub use buf_stream::*;
pub use stream_slice::*;
pub use io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};