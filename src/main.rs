use std::fs;

use unix_1972_bits::tap::Entry;

fn main() {
    let s2 = fs::read("s2-bits").unwrap();
    for chunk in s2.chunks_exact(64) {
        if let Some(entry) = Entry::parse(chunk.try_into().unwrap()) {
            println!("{entry:?}");
        }
    }
}
