use std::{collections::HashMap, fmt};

use anyhow::Result;

use crate::{debug::Bytes, interval::IntervalSet, s1::FileSegment};

pub struct Segmenter<'a> {
    tape: &'a [u8],
    block_size: usize,
    prev_block: &'a [u8],
    pub segments: Vec<Segment<'a>>,
    pub headers: HashMap<usize, FileSegment>,
    header_intervals: IntervalSet,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Segment<'a> {
    pub data: &'a [u8],
    pub offset: usize,
    pub kind: SegmentKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentKind {
    Original,
    Copy,
    AllNul,
    AllFF,
}

impl<'a> Segmenter<'a> {
    pub fn new(tape: &'a [u8], block_size: usize) -> Self {
        Segmenter {
            tape,
            block_size,
            prev_block: &[],
            segments: Vec::new(),
            headers: HashMap::new(),
            header_intervals: IntervalSet::new(0..tape.len()),
        }
    }

    pub fn add_header(&mut self, header: FileSegment) -> Result<()> {
        self.header_intervals.insert(header.range())?;
        assert!(self.headers.insert(header.offset, header).is_none());
        Ok(())
    }

    /// Partitions a tape into segments which are likely to be files.
    ///
    /// It exploits the behavior of the dumping program that was used, which
    /// evidently read files using a 512-byte buffer and directly dumped the
    /// buffer. When the buffer is not completely filled, as happens for the
    /// last block of a file, its tail is left unchanged, so by comparing blocks
    /// for common tails, file boundaries can be quite accurately identified.
    pub fn segment_blocks(&mut self) {
        let mut segment_start = 0;

        for block_start in (0..self.tape.len()).step_by(self.block_size) {
            let block_end = (block_start + self.block_size).min(self.tape.len());
            let block = &self.tape[block_start..block_end];

            // Check for all NULL or all 0xFF.
            let special = if block.iter().all(|&b| b == 0) {
                Some(SegmentKind::AllNul)
            } else if block.iter().all(|&b| b == 0xFF) {
                Some(SegmentKind::AllFF)
            } else {
                None
            };
            if let Some(kind) = special {
                if segment_start != block_start {
                    self.segments.push(Segment {
                        data: &self.tape[segment_start..block_start],
                        offset: segment_start,
                        kind: SegmentKind::Original,
                    });
                }
                self.segments.push(Segment {
                    data: block,
                    offset: block_start,
                    kind,
                });
                segment_start = block_end;
                self.prev_block = block;
                continue;
            }

            // Check whether this block and the previous have a common suffix.
            if !self.prev_block.is_empty() {
                let mut eq_index = 0;
                for i in (0..block.len()).rev() {
                    if block[i] != self.prev_block[i] {
                        eq_index = i + 1;
                        break;
                    }
                }
                // Apparent copies of length 1 or 2 are usually false positives.
                if eq_index + 2 < block.len() {
                    let split = block_start + eq_index;
                    if segment_start != split {
                        self.segments.push(Segment {
                            data: &self.tape[segment_start..split],
                            offset: segment_start,
                            kind: SegmentKind::Original,
                        });
                    }
                    self.segments.push(Segment {
                        data: &self.tape[split..block_end],
                        offset: split,
                        kind: SegmentKind::Copy,
                    });
                    segment_start = block_end;
                }
            }

            self.prev_block = block;
        }

        if segment_start != self.tape.len() {
            self.segments.push(Segment {
                data: &self.tape[segment_start..],
                offset: segment_start,
                kind: SegmentKind::Original,
            });
        }
    }
}

impl fmt::Debug for Segment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Segment")
            .field("offset", &self.offset)
            .field("len", &self.data.len())
            .field("kind", &self.kind)
            .field("data", &Bytes(&self.data))
            .finish()
    }
}
