//! Segmentation of files in a tape.

#![warn(missing_docs)]

use std::{cmp::Ordering, fmt, ops::Range};

/// Tool for segmenting a tape into likely files.
pub struct Segmenter<'t> {
    tape: &'t [u8],
    /// The block size that the tape was written with.
    block_size: usize,
    /// Splits, sorted by offsets.
    splits: Vec<Split>,
}

/// A location in a tape at which a file can be split.
#[derive(Clone, PartialEq, Eq)]
pub struct Split {
    /// Range of offsets at which this split can be performed.
    offsets: Range<usize>,
    /// The split strategy.
    kind: SplitKind,
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
    pub fn new(tape: &'t [u8], block_size: usize) -> Self {
        Segmenter {
            tape,
            block_size,
            splits: Vec::new(),
        }
    }

    /// Performs all supported splits.
    pub fn split_all(&mut self) {
        self.split_blocks();
        self.split_ff_blocks();
        self.split_lf_residue();
        self.split_nul_residue();
    }

    /// Splits into blocks. This inserts split points at multiples of the block
    /// size, and detects when residue likely remains from the last block and
    /// inserts split ranges there.
    pub fn split_blocks(&mut self) {
        let was_empty = self.splits.is_empty();
        let mut block_start = 0;
        let mut prev_block: &[u8] = &[];

        while block_start < self.tape.len() {
            let block_end = (block_start + self.block_size).min(self.tape.len());
            let block = &self.tape[block_start..block_end];

            self.splits
                .push(Split::new_point(block_start, SplitKind::Block));

            let mut eq_index = 0;
            if prev_block.is_empty() {
                for i in (0..block.len()).rev() {
                    if block[i] != 0 {
                        eq_index = i + 1;
                        break;
                    }
                }
            } else {
                for i in (0..block.len()).rev() {
                    if block[i] != prev_block[i] {
                        eq_index = i + 1;
                        break;
                    }
                }
            }
            if eq_index != block.len() {
                self.splits.push(Split::new(
                    block_start + eq_index..block_end,
                    SplitKind::Residue,
                ));
            }

            block_start = block_end;
            prev_block = block;
        }

        if !was_empty {
            self.splits.sort();
        }
    }

    /// Inserts split points around runs of blocks of all 0xFF bytes.
    pub fn split_ff_blocks(&mut self) {
        let was_empty = self.splits.is_empty();
        let mut block_start = 0;
        let mut prev_ff = false;

        while block_start < self.tape.len() {
            let block_end = (block_start + self.block_size).min(self.tape.len());
            let all_ff = self.tape[block_start..block_end].iter().all(|&b| b == 0xFF);
            if all_ff != prev_ff {
                self.splits
                    .push(Split::new_point(block_start, SplitKind::FF));
            }
            prev_ff = all_ff;
            block_start = block_end;
        }
        if prev_ff {
            self.splits
                .push(Split::new_point(self.tape.len(), SplitKind::FF));
        }

        if !was_empty {
            self.splits.sort();
        }
    }

    /// Insert split ranges allowing line feeds at the start of residue to be
    /// included in files.
    pub fn split_lf_residue(&mut self) {
        let mut i = 0;
        while i < self.splits.len() {
            let split = &self.splits[i];
            if split.kind == SplitKind::Residue && self.tape[split.start()] == b'\n' {
                let mut start = split.start();
                // Always split such that the text is terminated with LF.
                if start != 0 && self.tape[start - 1] != b'\n' {
                    start += 1;
                }
                let mut end = split.end();
                for i in split.start() + 1..split.end() {
                    if self.tape[i] != b'\n' {
                        end = i;
                        break;
                    }
                }
                i += 1;
                self.splits
                    .insert(i, Split::new(start..end, SplitKind::LfResidue));
            }
            i += 1;
        }
        self.splits.sort();
    }

    /// Insert split ranges allowing NUL bytes at the start of residue to be
    /// included in files.
    pub fn split_nul_residue(&mut self) {
        let mut i = 0;
        while i < self.splits.len() {
            let split = &self.splits[i];
            if split.kind == SplitKind::Residue && self.tape[split.start()] == 0 {
                let mut end = split.end();
                for i in split.start() + 1..split.end() {
                    if self.tape[i] != 0 {
                        end = i;
                        break;
                    }
                }
                i += 1;
                self.splits
                    .insert(i, Split::new(split.start()..end, SplitKind::NulResidue));
            }
            i += 1;
        }
        self.splits.sort();
    }
}

impl Split {
    /// Creates a new split over the given range.
    pub fn new(offsets: Range<usize>, kind: SplitKind) -> Self {
        assert!(offsets.start <= offsets.end);
        Split { offsets, kind }
    }

    /// Creates a new split at the given offset.
    pub fn new_point(offset: usize, kind: SplitKind) -> Self {
        Split {
            offsets: offset..offset,
            kind,
        }
    }

    /// Range of offsets at which this split can be performed.
    pub fn offsets(&self) -> Range<usize> {
        self.offsets.clone()
    }

    /// The start offset of this split.
    pub fn start(&self) -> usize {
        self.offsets.start
    }

    /// The end offset of this split.
    pub fn end(&self) -> usize {
        self.offsets.end
    }

    /// The split strategy.
    pub fn kind(&self) -> SplitKind {
        self.kind
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
