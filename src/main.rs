use std::{
    ffi::OsStr,
    fs::{self, File},
    os::unix::ffi::OsStrExt,
    path::Path,
};

use unix_1972_bits::{
    block::{SegmentKind, Segmenter},
    debug::{BlockLen, Bytes},
    s1::FileSegment,
    tap::Header,
};

fn main() {
    let s1 = fs::read("s1-bits").unwrap();
    segment_tape(
        &s1,
        Some(Path::new("s1-segments.csv")),
        Path::new("s1-segments.tar"),
    );

    let s2 = fs::read("s2-bits").unwrap();
    segment_tape(&s2, None, Path::new("s2-segments.tar"));
    let mut tar = tar::Builder::new(File::create("s2-files.tar").unwrap());
    for chunk in s2.chunks_exact(64) {
        if let Some(file) = Header::parse(chunk.try_into().unwrap()) {
            let data = &s2[file.range()];
            tar.append(&file.to_tar_header(), data).unwrap();
        }
    }
}

fn segment_tape(tape: &[u8], csv_path: Option<&Path>, tar_path: &Path) {
    let mut segmenter = Segmenter::new(tape, 512);

    for chunk in tape.chunks_exact(64) {
        if let Some(h) = Header::parse(chunk.try_into().unwrap()) {
            let file = FileSegment {
                path: h.path().into(),
                offset: h.offset(),
                len: h.size() as _,
            };
            segmenter.add_header(file).unwrap();
        }
    }

    if let Some(csv_path) = csv_path {
        let mut csv = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .from_path(csv_path)
            .unwrap();
        for header in csv.deserialize() {
            segmenter.add_header(header.unwrap()).unwrap();
        }
    }

    segmenter.segment_blocks();
    let mut tar = tar::Builder::new(File::create(tar_path).unwrap());
    for segment in &segmenter.segments {
        let mut h = tar::Header::new_old();
        if let Some(file) = segmenter.headers.get(&segment.offset) {
            if file.len != segment.data.len() {
                eprintln!(
                    "segment {:?} at offset {} has length {}; expected {}",
                    Bytes(&file.path),
                    segment.offset,
                    BlockLen(segment.data.len()),
                    BlockLen(file.len),
                );
            }
            let path = file.path.strip_prefix(b"/").unwrap_or(&file.path);
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
