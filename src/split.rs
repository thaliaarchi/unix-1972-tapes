//! Segmentation of files in a tape.

#![warn(missing_docs)]

use std::{cmp::Ordering, fmt, ops::Range};

/// Tool for segmenting a tape into likely files.
pub struct Segmenter<'t> {
    tape: &'t [u8],
    /// Splits, sorted by offsets.
    splits: Vec<Split>,
}

/// A location in a tape at which a file can be split.
#[derive(Clone, PartialEq, Eq)]
pub struct Split {
    /// Range of offsets at which this split can be performed.
    pub offsets: Range<usize>,
    /// The split strategy.
    pub kind: SplitKind,
}

/// Split strategies for segmenting files in a tape.
///
/// Variants are ordered by priority with higher priority comparing greater.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SplitKind {
    /// A fixed-size block of the tape.
    Block,
    /// Data at the end of a block which is identical in the previous block.
    Residue,
    /// NULs at the start of a residue.
    NulResidue,
    /// line feeds at the start of a residue.
    LfResidue,
    /// The size of an a.out binary determined from its header. It seems to
    /// under-count sometimes, perhaps for object files with undefined external
    /// symbols.
    AOutSize,
    /// A block-aligned a.out or `#!` magic number.
    Magic,
    /// Blocks of all 0xFF bytes.
    FF,
}

/// Split direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitDir {
    /// The start of a file.
    Start,
    /// The end of a file.
    End,
    /// Either the start or end of a file.
    Both,
}

impl<'t> Segmenter<'t> {
    /// Creates a new segmenter for the given tape.
    pub fn new(tape: &'t [u8]) -> Self {
        Segmenter {
            tape,
            splits: Vec::new(),
        }
    }

    /// Splits into blocks. This inserts split points at multiples of the block
    /// size, and detects when residue likely remains from the last block and
    /// inserts split ranges there.
    pub fn split_blocks(&mut self, block_size: usize) {
        let was_empty = self.splits.is_empty();
        let mut block_start = 0;
        let mut prev_block: &[u8] = &[];

        while block_start < self.tape.len() {
            let block_end = (block_start + block_size).min(self.tape.len());
            let block = &self.tape[block_start..block_end];

            self.splits.push(Split {
                offsets: block_start..block_start,
                kind: SplitKind::Block,
            });

            let eq_index = if prev_block.is_empty() {
                block.iter().rev().position(|&b| b != 0).unwrap_or(0)
            } else {
                let mut eq_index = 0;
                for i in (0..block.len()).rev() {
                    if block[i] != prev_block[i] {
                        eq_index = i + 1;
                        break;
                    }
                }
                eq_index
            };
            if eq_index != block.len() {
                self.splits.push(Split {
                    offsets: block_start + eq_index..block_end,
                    kind: SplitKind::Residue,
                });
            }

            block_start = block_end;
            prev_block = block;
        }

        if !was_empty {
            self.splits.sort();
        }
    }

    /// Inserts split points around runs of blocks of all 0xFF bytes.
    pub fn split_ff_blocks(&mut self, block_size: usize) {
        let was_empty = self.splits.is_empty();
        let mut block_start = 0;
        let mut prev_ff = false;

        while block_start < self.tape.len() {
            let block_end = (block_start + block_size).min(self.tape.len());
            let all_ff = self.tape[block_start..block_end].iter().all(|&b| b == 0xFF);
            if all_ff != prev_ff {
                self.splits.push(Split {
                    offsets: block_start..block_start,
                    kind: SplitKind::FF,
                });
            }
            prev_ff = all_ff;
            block_start = block_end;
        }
        if prev_ff {
            self.splits.push(Split {
                offsets: self.tape.len()..self.tape.len(),
                kind: SplitKind::FF,
            });
        }

        if !was_empty {
            self.splits.sort();
        }
    }
}

impl PartialOrd for Split {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Split {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.offsets.start.cmp(&other.offsets.start))
            .then(self.offsets.end.cmp(&other.offsets.end))
            .then(self.kind.cmp(&other.kind))
    }
}

impl fmt::Debug for Split {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Split");
        if self.offsets.start == self.offsets.end {
            s.field("offset", &self.offsets.start);
        } else {
            s.field("offsets", &self.offsets);
        }
        s.field("kind", &self.kind);
        s.finish()
    }
}

impl SplitKind {
    /// Direction of the split.
    pub fn direction(self) -> SplitDir {
        match self {
            SplitKind::Block => SplitDir::Start,
            SplitKind::Residue => SplitDir::End,
            SplitKind::NulResidue => SplitDir::End,
            SplitKind::LfResidue => SplitDir::End,
            SplitKind::AOutSize => SplitDir::End,
            SplitKind::Magic => SplitDir::Both,
            SplitKind::FF => SplitDir::Both,
        }
    }
}
