use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{self, File},
    os::unix::ffi::OsStrExt,
    path::Path,
};

use unix_1972_bits::{
    detect::{Magic, is_text},
    segment::{SegmentHeader, SegmentKind, SegmentLen, Segmenter},
    tap::Header,
    util::{BlockLen, Bytes},
};

fn main() {
    let s1 = fs::read("s1-bits").unwrap();
    segment_tape(
        &s1,
        Some(Path::new("s1-segments.csv")),
        Path::new("s1-segments.tar"),
        false,
    );

    let s2 = fs::read("s2-bits").unwrap();
    segment_tape(&s2, None, Path::new("s2-segments.tar"), false);
    let mut tar = tar::Builder::new(File::create("s2-files.tar").unwrap());
    for chunk in s2.chunks_exact(64) {
        if let Some(file) = Header::parse(chunk.try_into().unwrap()) {
            let data = &s2[file.range()];
            tar.append(&file.to_tar_header(), data).unwrap();
        }
    }
}

fn segment_tape(tape: &[u8], csv_path: Option<&Path>, tar_path: &Path, include_residue: bool) {
    let mut segmenter = Segmenter::new(tape, 512);

    for chunk in tape.chunks_exact(64) {
        if let Some(h) = Header::parse(chunk.try_into().unwrap()) {
            let file = SegmentHeader {
                path: h.path().into(),
                offset: h.offset(),
                len: SegmentLen::Manual(h.size() as _),
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
    let mut i = 0;
    while i < segmenter.segments().len() {
        let segment = &segmenter.segments()[i];
        let mut h = tar::Header::new_old();
        if let Some(file) = segmenter.header_for_offset(segment.offset) {
            if let SegmentLen::Manual(len) = file.len
                && len != segment.data.len()
            {
                eprintln!(
                    "segment {:?} at offset {} has length {}; expected {}",
                    Bytes(&file.path),
                    segment.offset,
                    BlockLen(segment.data.len()),
                    BlockLen(len),
                );
            }
            let path = file.path.strip_prefix(b"/").unwrap_or(&file.path);
            h.set_path(OsStr::from_bytes(path)).unwrap();
        } else {
            let ext = if is_text(segment.data) { "txt" } else { "bin" };
            let kind = match segment.kind {
                SegmentKind::Original => "",
                SegmentKind::Residue => ".copy",
                SegmentKind::AllNul => ".nul",
                SegmentKind::AllFF => ".ff",
            };
            let path = format!("segments/{}{kind}.{ext}", segment.offset);
            h.set_path(path).unwrap();
        }
        println!(
            "offset {:6} | len {:5} | {:8} | {} | {:11} | {:?}",
            segment.offset,
            segment.data.len(),
            format!("{:?}", segment.kind),
            if is_text(segment.data) {
                "text"
            } else {
                "bin "
            },
            Magic::detect(segment.data)
                .map(|m| format!("{m:?}"))
                .unwrap_or("none".to_owned()),
            Bytes(&h.path_bytes()),
        );
        let data = if include_residue
            && segment.kind == SegmentKind::Original
            && let Some(next_segment) = segmenter.segments().get(i + 1)
            && next_segment.kind == SegmentKind::Residue
        {
            i += 1;
            const DELIM: &[u8] = b"[SPLIT]";
            let mut data =
                Vec::with_capacity(segment.data.len() + DELIM.len() + next_segment.data.len());
            data.extend_from_slice(segment.data);
            data.extend_from_slice(DELIM);
            data.extend_from_slice(next_segment.data);
            Cow::Owned(data)
        } else {
            Cow::Borrowed(segment.data)
        };
        h.set_mode(0o644);
        h.set_size(data.len() as _);
        h.set_cksum();
        tar.append(&h, &*data).unwrap();
        i += 1;
    }
}
