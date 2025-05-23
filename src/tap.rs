use std::{
    fmt::{self, Write},
    mem,
};

#[derive(Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Entry {
    pub name: [u8; 32],
    pub mode: u8,
    pub uid: u8,
    pub size: [u8; 2],
    pub tmod: [u8; 4],
    pub taddress: [u8; 2],
    pub unused: [u8; 20],
    pub checksum: [u8; 2],
}

impl Entry {
    pub fn parse(raw: &[u8; 64]) -> Option<&Self> {
        let entry: &Entry = raw.into();
        if !raw.iter().all(|&b| b == 0) && entry.valid() {
            Some(entry)
        } else {
            None
        }
    }

    pub fn valid(&self) -> bool {
        let entry = unsafe { mem::transmute::<_, &[u8; 64]>(self) };
        let mut cksum = 0u16;
        for x in entry.chunks_exact(2) {
            cksum = cksum.wrapping_add(u16::from_le_bytes(x.try_into().unwrap()));
        }
        cksum == 0
    }

    pub fn name(&self) -> &[u8] {
        let mut name = &self.name[..];
        while let Some((&0, rest)) = name.split_last() {
            name = rest;
        }
        name
    }
    pub fn size(&self) -> u16 {
        u16::from_le_bytes(self.size)
    }
    pub fn taddress(&self) -> u16 {
        u16::from_le_bytes(self.taddress)
    }
    pub fn checksum(&self) -> u16 {
        u16::from_le_bytes(self.checksum)
    }
}

impl From<[u8; 64]> for Entry {
    fn from(raw: [u8; 64]) -> Self {
        unsafe { mem::transmute(raw) }
    }
}
impl From<&[u8; 64]> for &Entry {
    fn from(raw: &[u8; 64]) -> Self {
        unsafe { mem::transmute(raw) }
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Entry");
        s.field("name", &Bytes(self.name()));
        s.field("mode", &Mode(self.mode));
        s.field("uid", &self.uid);
        s.field("size", &self.size());
        s.field("tmod", &self.tmod);
        s.field("taddress", &self.taddress());
        if !self.unused.iter().all(|&b| b == 0) {
            s.field("unused", &Bytes(&self.unused));
        }
        s.field("checksum", &self.checksum());
        s.finish()
    }
}

struct Bytes<'a>(&'a [u8]);

impl fmt::Debug for Bytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        for (i, &b) in self.0.iter().enumerate() {
            match b {
                b'\\' => f.write_str("\\"),
                b'"' => f.write_str("\\\""),
                b' '..=b'~' => f.write_char(b as char),
                b'\t' => f.write_str("\\t"),
                b'\n' => f.write_str("\\n"),
                b'\r' => f.write_str("\\r"),
                0 if !matches!(self.0.get(i + 1), Some(b'0'..=b'9')) => f.write_str("\\0"),
                _ => write!(f, "\\{b:03o}"),
            }?;
        }
        f.write_char('"')
    }
}

struct Mode(u8);

impl fmt::Debug for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:03o}", self.0)
    }
}
