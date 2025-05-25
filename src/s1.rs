use std::{fmt, ops::Range};

use anyhow::Result;
use serde::Deserialize;

use crate::{debug::Bytes, interval::IntervalSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segments<'s> {
    data: &'s [u8],
    pub segments: Vec<FileSegment>,
    pub intervals: IntervalSet,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub struct FileSegment {
    #[serde(alias = "Path", with = "serde_bytes")]
    pub path: Vec<u8>,
    #[serde(alias = "Offset")]
    pub offset: usize,
    #[serde(alias = "Length", alias = "length")]
    pub len: usize,
}

impl<'a> Segments<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Segments {
            data,
            segments: Vec::new(),
            intervals: IntervalSet::new(0..data.len()),
        }
    }

    pub fn insert(&mut self, segment: FileSegment) -> Result<()> {
        self.intervals.insert(segment.range())?;
        self.segments.push(segment);
        Ok(())
    }
}

impl FileSegment {
    pub fn range(&self) -> Range<usize> {
        self.offset..self.offset + self.len
    }
}

impl fmt::Debug for FileSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileSegment")
            .field("path", &Bytes(&self.path))
            .field("offset", &self.offset)
            .field("len", &self.len)
            .finish()
    }
}
