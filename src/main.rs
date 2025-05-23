use std::fs::{self, File};

use unix_1972_bits::tap::Entry;

fn main() {
    let s2 = fs::read("s2-bits").unwrap();

    let mut tar = tar::Builder::new(File::create("s2-out.tar").unwrap());

    for (i, chunk) in s2.chunks_exact(64).enumerate() {
        if let Some(entry) = Entry::parse(chunk.try_into().unwrap()) {
            let start = i * 64;
            println!("{start}: {entry:?}");
            let data = &s2[entry.range()];
            tar.append(&entry.to_tar_header(), data).unwrap();
        }
    }

    let s1 = fs::read("s1-bits").unwrap();

    let mut tar = tar::Builder::new(File::create("s1-out.tar").unwrap());

    for (i, block) in s1.chunks(512).enumerate() {
        let mut h = tar::Header::new_old();
        h.set_path(format!("block{i}")).unwrap();
        h.set_mode(0o644);
        h.set_size(block.len() as _);
        h.set_cksum();
        tar.append(&h, block).unwrap();
    }
}
