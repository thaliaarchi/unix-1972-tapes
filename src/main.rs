use std::fs::{self, File};

use unix_1972_bits::{block::segment_blocks, s1::Segments, tap::Entry};

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
    let mut segments = Segments::new(&s1);
    for res in csv.deserialize() {
        segments.insert(res.unwrap()).unwrap();
    }

    let mut tar = tar::Builder::new(File::create("s1-segments.tar").unwrap());
    for segment in segments.segments {
        let data = &s1[segment.range()];
        let mut h = tar::Header::new_old();
        let path = segment.path.strip_prefix("/").unwrap_or(&segment.path);
        h.set_path(format!("segments/{path}")).unwrap();
        h.set_mode(0o644);
        h.set_size(data.len() as _);
        h.set_cksum();
        tar.append(&h, data).unwrap();
    }
    let mut chunks = Vec::new();
    for (i, block) in s1.chunks(512).enumerate() {
        let range = i * 512..i * 512 + block.len();
        chunks.clear();
        segments.intervals.get_disjoint(range.clone(), &mut chunks);
        for chunk in &chunks {
            let data = &s1[chunk.clone()];
            let typ = if data.iter().all(|&b| matches!(b, 0x07..=0x0f | b' '..=b'~')) {
                "txt"
            } else {
                "bin"
            };
            let offset = chunk.start - range.start;
            let path = if offset == 0 {
                format!("blocks/block{i}.{typ}")
            } else {
                format!("blocks/block{i}.{offset}.{typ}")
            };
            let mut h = tar::Header::new_old();
            h.set_path(path).unwrap();
            h.set_mode(0o644);
            h.set_size(chunk.len() as _);
            h.set_cksum();
            tar.append(&h, data).unwrap();
        }
    }
    println!("{:?}", segments.intervals);

    let segments = segment_blocks(&s1, 512);
    for segment in segments {
        println!("{segment:?}");
    }
}
