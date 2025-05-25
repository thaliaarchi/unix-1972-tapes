use std::{collections::HashMap, fmt, ops::Range};

use anyhow::Result;

use crate::{
    debug::{BlockLen, Bytes},
    interval::IntervalSet,
    s1::FileSegment,
};

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
        let mut block_start = 0;
        while block_start < self.tape.len() {
            let block_end = (block_start + self.block_size).min(self.tape.len());
            let block = &self.tape[block_start..block_end];

            // Check for blocks that are all NUL or all 0xFF.
            let uniform = if let Some(end) = self.check_uniform(block_start, 0) {
                Some((end, SegmentKind::AllNul))
            } else if let Some(end) = self.check_uniform(block_start, 0xFF) {
                Some((end, SegmentKind::AllFF))
            } else {
                None
            };
            if let Some((uniform_end, kind)) = uniform {
                if segment_start != block_start {
                    // Join NUL blocks surrounded by zeros.
                    if kind == SegmentKind::AllNul
                        && self.tape.get(block_start.wrapping_sub(1)) == Some(&0)
                        && self.tape.get(uniform_end) == Some(&0)
                    {
                        block_start = uniform_end;
                        continue;
                    }
                    self.push(segment_start..block_start, SegmentKind::Original);
                }
                self.push(block_start..uniform_end, kind);
                self.prev_block = &self.tape[uniform_end - self.block_size..uniform_end];
                segment_start = uniform_end;
                block_start = uniform_end;
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
                        self.push(segment_start..split, SegmentKind::Original);
                    }
                    self.push(split..block_end, SegmentKind::Copy);
                    segment_start = block_end;
                }
            }

            self.prev_block = block;
            block_start += self.block_size;
        }

        if segment_start != self.tape.len() {
            self.push(segment_start..self.tape.len(), SegmentKind::Original);
        }
    }

    fn check_uniform(&mut self, block_start: usize, byte: u8) -> Option<usize> {
        let mut end = block_start;
        for block in self.tape[block_start..].chunks_exact(self.block_size) {
            if !block.iter().all(|&b| b == byte) {
                break;
            }
            end += self.block_size;
            if self.headers.contains_key(&end) {
                break;
            }
        }
        (end != block_start).then_some(end)
    }

    #[track_caller]
    fn push(&mut self, range: Range<usize>, kind: SegmentKind) {
        self.segments.push(Segment {
            data: &self.tape[range.clone()],
            offset: range.start,
            kind,
        });
    }
}

impl fmt::Debug for Segment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Segment")
            .field("offset", &self.offset)
            .field("len", &BlockLen(self.data.len()))
            .field("kind", &self.kind)
            .field("data", &Bytes(&self.data))
            .finish()
    }
}
