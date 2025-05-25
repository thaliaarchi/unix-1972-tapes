use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    os::unix::ffi::OsStrExt,
};

use unix_1972_bits::{
    block::{SegmentKind, segment_blocks},
    debug::Bytes,
    interval::IntervalSet,
    s1::FileSegment,
    tap::Header,
};

fn main() {
    let s2 = fs::read("s2-bits").unwrap();

    let mut tar = tar::Builder::new(File::create("s2-files.tar").unwrap());
    for (i, chunk) in s2.chunks_exact(64).enumerate() {
        if let Some(file) = Header::parse(chunk.try_into().unwrap()) {
            let start = i * 64;
            println!("{start}: {file:?}");
            let data = &s2[file.range()];
            tar.append(&file.to_tar_header(), data).unwrap();
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
        let file: FileSegment = res.unwrap();
        intervals.insert(file.range()).unwrap();
        if by_offset.insert(file.offset, file).is_some() {
            panic!("duplicate offset");
        }
    }

    let segments = segment_blocks(&s1, 512);
    let mut tar = tar::Builder::new(File::create("s1-segments.tar").unwrap());
    for segment in segments {
        let mut h = tar::Header::new_old();
        if let Some(named) = by_offset.remove(&segment.offset) {
            if named.len != segment.data.len() {
                eprintln!(
                    "segment {:?} at offset {} has length {}; expected {}",
                    Bytes(&named.path),
                    segment.offset,
                    segment.data.len(),
                    named.len,
                );
            }
            let path = (named.path.strip_prefix(b"/")).unwrap_or(&named.path);
            h.set_path(OsStr::from_bytes(path)).unwrap();
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
            let path = format!("segments/{}{kind}.{ext}", segment.offset);
            h.set_path(path).unwrap();
        }
        h.set_mode(0o644);
        h.set_size(segment.data.len() as _);
        h.set_cksum();
        tar.append(&h, segment.data).unwrap();
    }
}
