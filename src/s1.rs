use std::ops::Range;

use anyhow::Result;
use serde::Deserialize;

use crate::interval::IntervalSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segments<'s> {
    data: &'s [u8],
    pub segments: Vec<Segment>,
    pub intervals: IntervalSet,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Segment {
    #[serde(alias = "Path")]
    pub path: String,
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

    pub fn insert(&mut self, segment: Segment) -> Result<()> {
        self.intervals.insert(segment.range())?;
        self.segments.push(segment);
        Ok(())
    }
}

impl Segment {
    pub fn range(&self) -> Range<usize> {
        self.offset..self.offset + self.len
    }
}
