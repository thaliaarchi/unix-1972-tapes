use std::{
    collections::HashMap,
    fs::{self, File},
};

use unix_1972_bits::{
    block::{SegmentKind, segment_blocks},
    interval::IntervalSet,
    s1::Segment,
    tap::Entry,
};

fn main() {
    let s2 = fs::read("s2-bits").unwrap();

    let mut tar = tar::Builder::new(File::create("s2-files.tar").unwrap());
    for (i, chunk) in s2.chunks_exact(64).enumerate() {
        if let Some(entry) = Entry::parse(chunk.try_into().unwrap()) {
            let start = i * 64;
            println!("{start}: {entry:?}");
            let data = &s2[entry.range()];
            tar.append(&entry.to_tar_header(), data).unwrap();
        }
    }

    let s1 = fs::read("s1-bits").unwrap();

    let mut csv = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_path("s1-segments.csv")
        .unwrap();
    let mut intervals = IntervalSet::new(0..s1.len());
    let mut by_offset = HashMap::new();
    for res in csv.deserialize() {
        let segment: Segment = res.unwrap();
        intervals.insert(segment.range()).unwrap();
        if by_offset.insert(segment.offset, segment).is_some() {
            panic!("duplicate offset");
        }
    }

    let segments = segment_blocks(&s1, 512);
    let mut tar = tar::Builder::new(File::create("s1-segments.tar").unwrap());
    for segment in segments {
        let mut h = tar::Header::new_old();
        let path = if let Some(named) = by_offset.remove(&segment.offset) {
            if named.len != segment.data.len() {
                eprintln!(
                    "segment {} at offset {} has length {}; expected {}",
                    named.path,
                    segment.offset,
                    segment.data.len(),
                    named.len,
                );
            }
            if named.path == "copy" {
                continue;
            }
            let path = named.path.strip_prefix("/").unwrap_or(&named.path);
            format!("files/{path}")
        } else {
            let ext = if segment
                .data
                .iter()
                .all(|&b| matches!(b, 0x07..=0x0f | b' '..=b'~'))
            {
                "txt"
            } else {
                "bin"
            };
            let kind = match segment.kind {
                SegmentKind::Original => "",
                SegmentKind::Copy => ".copy",
                SegmentKind::AllNul => ".nul",
                SegmentKind::AllFF => ".ff",
            };
            format!("segments/{}{kind}.{ext}", segment.offset)
        };
        h.set_path(path).unwrap();
        h.set_mode(0o644);
        h.set_size(segment.data.len() as _);
        h.set_cksum();
        tar.append(&h, segment.data).unwrap();
    }
}
