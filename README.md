# Unix 1972 tapes

Analysis and parsing of the s1 and s2 tapes from [1972_stuff](https://www.tuhs.org/Archive/Distributions/Research/1972_stuff/)
in the Unix Archive.

The s2 tape is in the `tap` format, which is simple to decode and it has already
been analyzed.

The s1 tape is a self-unpacking tape which loads the cold kernel so you can
unpack the rest of the system from s2 with `tap`. Yufeng Gao has reconstructed a
disk image of the full system by running it as intended in SIMH
[[TUHS](https://www.tuhs.org/pipermail/tuhs/2025-February/031420.html),
[GitHub](https://github.com/TheBrokenPipe/Research-UNIX-V2-Beta)]. I am reverse
engineering the unpacking code to be able to analyze its behavior and account
for every byte in the tape (see, e.g., [s1/block0.s](s1/block0.s)).

Even without loading s1, its structure implies the sizes of its contents. I
observed that files are written in 512-byte blocks and when a file is not a
multiple of 512 in length, its last block will retain the data from the previous
block in its tail (“residue”). Since it's unlikely that the same byte will
appear at the end of a file and at the same position in the same block, a long
enough common suffix is a strong indicator that the file has ended. I handle
edge cases like with ensuring line feeds at the end of a text file or taking NUL
bytes from the residue for binary files. It's not perfect, but these heuristics
yield segments that are likely to be files. Currently, the Rust code is quite
messy.
