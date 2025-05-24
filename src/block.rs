use std::fmt;

use crate::debug::Bytes;

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

pub fn segment_blocks<'a>(data: &'a [u8], block_size: usize) -> Vec<Segment<'a>> {
    let mut segments = Vec::new();

    let mut prev = &[][..];
    let data_ptr = data.as_ptr() as usize;
    let mut start_offset = 0;

    for block in data.chunks(block_size) {
        let ptrs = block.as_ptr_range();
        let offsets = ptrs.start as usize - data_ptr..ptrs.end as usize - data_ptr;

        // Check for all NULL or all 0xFF.
        let special = if block.iter().all(|&b| b == 0) {
            Some(SegmentKind::AllNul)
        } else if block.iter().all(|&b| b == 0xFF) {
            Some(SegmentKind::AllFF)
        } else {
            None
        };
        if let Some(kind) = special {
            if start_offset != offsets.start {
                segments.push(Segment {
                    data: &data[start_offset..offsets.start],
                    offset: start_offset,
                    kind: SegmentKind::Original,
                });
            }
            segments.push(Segment {
                data: block,
                offset: offsets.start,
                kind,
            });
            start_offset = offsets.end;
            prev = block;
            continue;
        }

        // Check whether this block and the previous have a common suffix.
        if !prev.is_empty() {
            let mut eq_index = 0;
            for i in (0..block.len()).rev() {
                if block[i] != prev[i] {
                    eq_index = i + 1;
                    break;
                }
            }
            if eq_index != block.len() {
                let split = offsets.start + eq_index;
                if start_offset != split {
                    segments.push(Segment {
                        data: &data[start_offset..split],
                        offset: start_offset,
                        kind: SegmentKind::Original,
                    });
                }
                segments.push(Segment {
                    data: &data[split..offsets.end],
                    offset: split,
                    kind: SegmentKind::Copy,
                });
                start_offset = offsets.end;
            }
        }

        prev = block;
    }

    if start_offset != data.len() {
        segments.push(Segment {
            data: &data[start_offset..],
            offset: start_offset,
            kind: SegmentKind::Original,
        });
    }

    segments
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
