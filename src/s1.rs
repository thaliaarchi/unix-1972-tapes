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
    pub path: String,
    pub block: usize,
    #[serde(flatten)]
    pub range: SegmentRange,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum SegmentRange {
    Block { end: usize },
    Offset { offset: usize, len: Option<usize> },
    Chunk { len: usize },
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

    pub fn insert_json(&mut self, json: &str) -> Result<()> {
        let start_len = self.segments.len();
        let mut segments: Vec<Segment> = json5::from_str(json)?;
        self.segments.append(&mut segments);
        for segment in &self.segments[start_len..] {
            if let Err(e) = self.intervals.insert(segment.range()) {
                self.segments.truncate(start_len);
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Segment {
    pub fn range(&self) -> Range<usize> {
        let mut start = self.block * 512;
        let end = match self.range {
            SegmentRange::Block { end } => end * 512,
            SegmentRange::Offset { offset, len } => {
                start += offset;
                match len {
                    Some(len) => start + len,
                    None => (self.block + offset.div_ceil(512)) * 512,
                }
            }
            SegmentRange::Chunk { len } => start + len,
        };
        assert!(start <= end);
        start..end
    }
}
