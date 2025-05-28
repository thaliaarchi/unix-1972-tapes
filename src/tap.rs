//! tap file decoding for s2-bits.
//!
//! Follows the logic of tapfs from Plan 9 ([dmr's email](https://www.tuhs.org/Archive/Distributions/Research/1972_stuff/dmr_plugin),
//! Plan 9 from User Space [source](https://github.com/9fans/plan9port/blob/master/src/cmd/tapefs/tapfs.c)
//! and [manpage](https://plan9.io/magic/man2html/4/tapefs)) and the V1 stat
//! translation in [Apout](https://github.com/DoctorWkt/Apout/blob/master/v1trap.c).

#![warn(missing_docs)]

use std::{ffi::OsStr, fmt, mem, ops::Range, os::unix::ffi::OsStrExt, time::Duration, u32};

use jiff::{Timestamp, civil::Date, tz::TimeZone};

use crate::util::{Bytes, U16Le, U32Me};

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
    pub size: U16Le,
    /// The modification time in Unix V1 format.
    pub mtime: U32Me,
    /// The index of the 512-byte block which the file contents start at.
    pub block: U16Le,
    /// Unused padding.
    pub unused: [u8; 20],
    /// The checksum of this header.
    pub cksum: U16Le,
}

/// Permission bits in the Unix V1 format.
pub struct Mode(pub u8);

/// Timestamp in the Unix V1 format, i.e., 1/60 seconds since an [epoch](Epoch).
///
/// When an epoch is not specified, it defaults to 1972.
///
/// > In the early version of UNIX, timestamps were in 1/60th second units. A
/// > 32-bit counter using these units overflows in 2.5 years, so the epoch had
/// > to be changed periodically, and I believe 1970, 1971, 1972 and 1973 were
/// > all epochs at one stage or another.
/// >
/// > Given that the C compiler passes, and the library, are dated in June of
/// > the epoch year, and that Dennis has said ``1972-73 were the truly
/// > formative years in the development of the C language'', it's therefore
/// > unlikely that the epoch for the s2 tape is 1971: it is more likely to be
/// > 1972. The tape also contains several 1st Edition a.out binaries, which
/// > also makes it unlikely to be 1973.
/// >
/// > [[Warren Toomey](https://www.tuhs.org/Archive/Distributions/Research/1972_stuff/Readme)]
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
        self.size.get()
    }

    /// The modification time in the Unix V1 format.
    pub fn mtime(&self) -> Time {
        Time(self.mtime.get())
    }

    /// The index of the 512-byte block which the file contents start at.
    pub fn block(&self) -> u16 {
        self.block.get()
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
        self.cksum.get()
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
        epoch.timestamp() + since
    }

    /// The time as a timestamp with seconds resolution in the given epoch.
    pub fn timestamp_seconds(&self, epoch: Epoch) -> Timestamp {
        epoch.timestamp() + Duration::from_secs((self.0 / 60) as _)
    }

    /// The number of seconds since the 1970 Unix epoch.
    pub fn seconds(&self, epoch: Epoch) -> u32 {
        self.timestamp_seconds(epoch).as_second() as u32
    }

    /// The number of 1/60ths of a second in this time.
    pub fn subseconds(&self) -> u8 {
        (self.0 % 60) as _
    }
}

/// Formats the time relative to an epoch specified in the precision field, or
/// 1972 if not given, along with the raw time integer.
impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)?;
        write!(f, " ({})", self.0)
    }
}

/// Formats the time relative to an epoch specified in the precision field, or
/// 1972 if not given.
impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let epoch = match f.precision() {
            Some(1970) => Epoch::Y1970,
            Some(1971) => Epoch::Y1971,
            Some(1972) => Epoch::Y1972,
            Some(1973) => Epoch::Y1973,
            Some(_) => return Err(fmt::Error),
            None => Epoch::Y1972,
        };
        let t = self.timestamp_seconds(epoch).strftime("%F %T");
        write!(f, "{t}:{:02}", self.subseconds())
    }
}

impl Epoch {
    /// The epoch as a timestamp.
    pub fn timestamp(self) -> Timestamp {
        Date::constant(1970 + self as i16, 1, 1)
            .to_zoned(TimeZone::UTC)
            .unwrap()
            .timestamp()
    }
}

#[test]
fn time_seconds_range() {
    let min = Time(0).timestamp(Epoch::Y1970).as_second();
    assert_eq!(min, 0);
    let max = Time(u32::MAX).timestamp(Epoch::Y1973).as_second();
    assert!(max < u32::MAX as i64);
}

#[test]
fn epoch_seconds_since_1970() {
    // Constants used by Apout for seconds from 1970 to 1971 and 1972.
    // https://github.com/DoctorWkt/Apout/blob/e88a446ace064f5a41e1a47d9ae8278b83b27a20/v1trap.c#L142-L143
    assert_eq!(Epoch::Y1971.timestamp().as_second(), 31536000);
    assert_eq!(Epoch::Y1972.timestamp().as_second(), 63072000);
}

#[test]
fn time_format() {
    assert_eq!(format!("{:?}", Time(0)), "1972-01-01 00:00:00:00 (0)");
    assert_eq!(
        format!("{:.1970?}", Time(11)),
        "1970-01-01 00:00:00:11 (11)",
    );
    assert_eq!(
        format!("{:.1971?}", Time(22)),
        "1971-01-01 00:00:00:22 (22)",
    );
    assert_eq!(
        format!("{:.1972?}", Time(33)),
        "1972-01-01 00:00:00:33 (33)",
    );
    assert_eq!(
        format!("{:.1973?}", Time(44)),
        "1973-01-01 00:00:00:44 (44)",
    );
}
