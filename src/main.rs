use std::fs::{self, File};

use unix_1972_bits::tap::Entry;

fn main() {
    let s2 = fs::read("s2-bits").unwrap();

    let w = File::create("out.tar").unwrap();
    let mut tar = tar::Builder::new(w);

    for (i, chunk) in s2.chunks_exact(64).enumerate() {
        if let Some(entry) = Entry::parse(chunk.try_into().unwrap()) {
            let start = i * 64;
            println!("{start}: {entry:?}");
            let data = &s2[entry.range()];
            tar.append(&entry.to_tar_header(), data).unwrap();
        }
    }
}
