//! Segmentation of files in a tape.

#![warn(missing_docs)]

/// Split strategies for segmenting files in a tape.
///
/// Variants are ordered by priority with higher priority comparing greater.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Split {
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

impl Split {
    /// Direction of the split.
    pub fn direction(self) -> SplitDir {
        match self {
            Split::Block => SplitDir::Start,
            Split::Residue => SplitDir::End,
            Split::NulResidue => SplitDir::End,
            Split::LfResidue => SplitDir::End,
            Split::AOutSize => SplitDir::End,
            Split::Magic => SplitDir::Both,
            Split::FF => SplitDir::Both,
        }
    }
}
