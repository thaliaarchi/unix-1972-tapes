//! tap file decoding for s2-bits.
//!
//! Follows the logic of tapfs from Plan 9 ([dmr's email](https://www.tuhs.org/Archive/Distributions/Research/1972_stuff/dmr_plugin),
//! Plan 9 from User Space [source](https://github.com/9fans/plan9port/blob/master/src/cmd/tapefs/tapfs.c)
//! and [manpage](https://plan9.io/magic/man2html/4/tapefs)) and the V1 stat
//! translation in [Apout](https://github.com/DoctorWkt/Apout/blob/master/v1trap.c).

#![warn(missing_docs)]

use std::{ffi::OsStr, fmt, mem, ops::Range, os::unix::ffi::OsStrExt, time::Duration, u32};

use jiff::{Timestamp, civil::Date, tz::TimeZone};

use crate::debug::Bytes;

/// A file header in a tap file.
#[derive(Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Header {
    /// The file path.
    pub path: [u8; 32],
    /// The permission bits.
    pub mode: u8,
    /// The user ID.
    pub uid: u8,
    /// The length of the file contents.
    pub size: [u8; 2],
    /// The modification time in Unix V1 format.
    pub mtime: [u8; 4],
    /// The index of the 512-byte block which the file contents start at.
    pub block: [u8; 2],
    /// Unused padding.
    pub unused: [u8; 20],
    /// The checksum of this header.
    pub cksum: [u8; 2],
}

/// Permission bits in the Unix V1 format.
pub struct Mode(pub u8);

/// Timestamp in the Unix V1 format, i.e., 1/60 seconds since an [epoch](Epoch).
///
/// > In the early version of UNIX, timestamps were in 1/60th second units. A
/// > 32-bit counter using these units overflows in 2.5 years, so the epoch had
/// > to be changed periodically, and I believe 1970, 1971, 1972 and 1973 were
/// > all epochs at one stage or another. [[Warren Toomey](https://www.tuhs.org/Archive/Distributions/Research/1972_stuff/Readme)]
pub struct Time(pub u32);

/// The epoch of a Unix V1 timestamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Epoch {
    /// 1970 epoch.
    Y1970 = 0,
    /// 1971 epoch.
    Y1971 = 1,
    /// 1972 epoch.
    Y1972 = 2,
    /// 1973 epoch.
    Y1973 = 3,
}

impl Header {
    /// Parses a file header from a tap file.
    pub fn parse(raw: &[u8; 64]) -> Option<&Self> {
        let header: &Header = raw.into();
        if !raw.iter().all(|&b| b == 0) && header.valid() {
            Some(header)
        } else {
            None
        }
    }

    /// Validates the header against its checksum.
    pub fn valid(&self) -> bool {
        let bytes = unsafe { mem::transmute::<_, &[u8; 64]>(self) };
        let mut cksum = 0u16;
        for x in bytes.chunks_exact(2) {
            cksum = cksum.wrapping_add(u16::from_le_bytes(x.try_into().unwrap()));
        }
        cksum == 0
    }

    /// The file path.
    pub fn path(&self) -> &[u8] {
        let mut path = &self.path[..];
        while let Some((&0, rest)) = path.split_last() {
            path = rest;
        }
        path
    }

    /// The permission bits.
    pub fn mode(&self) -> Mode {
        Mode(self.mode)
    }

    /// The length of the file contents.
    pub fn size(&self) -> u16 {
        u16::from_le_bytes(self.size)
    }

    /// The modification time in the Unix V1 format.
    pub fn mtime(&self) -> Time {
        let t = self.mtime;
        Time((t[1] as u32) << 24 | (t[0] as u32) << 16 | (t[3] as u32) << 8 | (t[2] as u32) << 0)
    }

    /// The index of the 512-byte block which the file contents start at.
    pub fn block(&self) -> u16 {
        u16::from_le_bytes(self.block)
    }

    /// The byte offset in the tap file of the start of the file.
    pub fn offset(&self) -> usize {
        (self.block() as usize) * 512
    }

    /// The byte offsets in the tap file of the file contents.
    pub fn range(&self) -> Range<usize> {
        let offset = self.offset();
        offset..offset + self.size() as usize
    }

    /// The checksum of this header.
    pub fn cksum(&self) -> u16 {
        u16::from_le_bytes(self.cksum)
    }

    /// Converts this tap header to a tar header.
    pub fn to_tar_header(&self) -> tar::Header {
        let mut h = tar::Header::new_old();
        h.set_path(OsStr::from_bytes(&self.path()[1..])).unwrap();
        h.set_mode(self.mode().to_posix() as _);
        h.set_uid(self.uid as _);
        h.set_size(self.size() as _);
        h.set_mtime(self.mtime().seconds(Epoch::Y1972) as _);
        h.set_cksum();
        h
    }
}

impl From<[u8; 64]> for Header {
    fn from(raw: [u8; 64]) -> Self {
        unsafe { mem::transmute(raw) }
    }
}
impl From<&[u8; 64]> for &Header {
    fn from(raw: &[u8; 64]) -> Self {
        unsafe { mem::transmute(raw) }
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Entry");
        s.field("path", &Bytes(self.path()));
        s.field("mode", &self.mode());
        s.field("uid", &self.uid);
        s.field("size", &self.size());
        s.field("mtime", &self.mtime());
        s.field("block", &self.block());
        if !self.unused.iter().all(|&b| b == 0) {
            s.field("unused", &Bytes(&self.unused));
        }
        s.field("cksum", &self.cksum());
        s.finish()
    }
}

#[rustfmt::skip]
#[allow(dead_code)]
mod mode {
    pub const V1_SET_UID: u8     = 0o40;
    pub const V1_EXEC: u8        = 0o20;
    pub const V1_OWNER_READ: u8  = 0o10;
    pub const V1_OWNER_WRITE: u8 = 0o04;
    pub const V1_WORLD_READ: u8  = 0o02;
    pub const V1_WORLD_WRITE: u8 = 0o01;

    pub const POSIX_SET_UID: u16     = 0o004000;
    pub const POSIX_SET_GID: u16     = 0o002000;
    pub const POSIX_STICKY: u16      = 0o001000;
    pub const POSIX_OWNER_READ: u16  = 0o000400;
    pub const POSIX_OWNER_WRITE: u16 = 0o000200;
    pub const POSIX_OWNER_EXEC: u16  = 0o000100;
    pub const POSIX_GROUP_READ: u16  = 0o000040;
    pub const POSIX_GROUP_WRITE: u16 = 0o000020;
    pub const POSIX_GROUP_EXEC: u16  = 0o000010;
    pub const POSIX_OTHER_READ: u16  = 0o000004;
    pub const POSIX_OTHER_WRITE: u16 = 0o000002;
    pub const POSIX_OTHER_EXEC: u16  = 0o000001;
}

impl Mode {
    /// Converts a POSIX mode to V1.
    pub fn from_posix(mode: u16) -> Mode {
        use mode::*;
        let (posix, mut v1) = (mode, 0);
        if posix & POSIX_SET_UID != 0 {
            v1 |= V1_SET_UID;
        }
        if posix & (POSIX_OWNER_EXEC | POSIX_GROUP_EXEC | POSIX_OTHER_EXEC) != 0 {
            v1 |= V1_EXEC;
        }
        if posix & (POSIX_OWNER_READ) != 0 {
            v1 |= V1_OWNER_READ;
        }
        if posix & (POSIX_OWNER_WRITE) != 0 {
            v1 |= V1_OWNER_WRITE;
        }
        if posix & (POSIX_GROUP_READ | POSIX_OTHER_READ) != 0 {
            v1 |= V1_WORLD_READ;
        }
        if posix & (POSIX_GROUP_WRITE | POSIX_OTHER_WRITE) != 0 {
            v1 |= V1_WORLD_WRITE;
        }
        Mode(v1)
    }

    /// Converts a V1 mode to POSIX.
    pub fn to_posix(self) -> u16 {
        use mode::*;
        let (v1, mut posix) = (self.0, 0);
        if v1 & V1_SET_UID != 0 {
            posix |= POSIX_SET_UID;
        }
        if v1 & V1_EXEC != 0 {
            posix |= POSIX_OWNER_EXEC | POSIX_GROUP_EXEC | POSIX_OTHER_EXEC;
        }
        if v1 & V1_OWNER_READ != 0 {
            posix |= POSIX_OWNER_READ;
        }
        if v1 & V1_OWNER_WRITE != 0 {
            posix |= POSIX_OWNER_WRITE;
        }
        if v1 & V1_WORLD_READ != 0 {
            posix |= POSIX_GROUP_READ | POSIX_OTHER_READ;
        }
        if v1 & V1_WORLD_WRITE != 0 {
            posix |= POSIX_GROUP_WRITE | POSIX_OTHER_WRITE;
        }
        posix
    }
}

impl fmt::Debug for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:03o}", self.0)
    }
}

impl Time {
    /// The time as a timestamp in the given epoch.
    pub fn timestamp(&self, epoch: Epoch) -> Timestamp {
        let seconds = self.0 / 60;
        let frac = self.0 % 60;
        let since = Duration::new(seconds as _, (frac as u64 * 1_000_000_000 / 60) as _);

        let epoch = Date::constant(1970 + epoch as i16, 1, 1);
        let epoch = epoch.to_zoned(TimeZone::UTC).unwrap();

        epoch.timestamp() + since
    }

    /// The number of seconds since the 1970 Unix epoch.
    pub fn seconds(&self, epoch: Epoch) -> u32 {
        self.timestamp(epoch).as_second() as u32
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0, self.timestamp(Epoch::Y1972))
    }
}

#[test]
fn time_seconds_range() {
    let min = Time(0).timestamp(Epoch::Y1970).as_second();
    assert_eq!(min, 0);
    let max = Time(u32::MAX).timestamp(Epoch::Y1973).as_second();
    assert!(max < u32::MAX as i64);
}
