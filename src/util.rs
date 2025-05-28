use std::fmt::{self, Write};

macro_rules! int_ty(($T:ident, $Int:ty, $N:literal, |$b:ident| $get:expr) => {
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct $T(pub [u8; $N]);

    impl $T {
        pub fn get(self) -> $Int {
            let $b = self.0;
            $get
        }
    }

    impl fmt::Debug for $T {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.get().fmt(f)
        }
    }
    impl fmt::Display for $T {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.get().fmt(f)
        }
    }


    impl From<$Int> for $T {
        fn from(value: $Int) -> Self {
            $T(value.to_le_bytes())
        }
    }
    impl From<$T> for $Int {
        fn from(value: $T) -> Self {
            value.get()
        }
    }
    impl From<[u8; $N]> for $T {
        fn from(value: [u8; $N]) -> Self {
            $T(value)
        }
    }

    impl PartialEq<$Int> for $T {
        fn eq(&self, other: &$Int) -> bool {
            self.get() == *other
        }
    }
    impl PartialEq<$T> for $Int {
        fn eq(&self, other: &$T) -> bool {
            *self == other.get()
        }
    }
});

int_ty!(U16Le, u16, 2, |b| u16::from_le_bytes(b));
int_ty!(U32Me, u32, 4, |b| u32::from_le_bytes([
    b[2], b[3], b[0], b[1]
]));

pub struct Bytes<'a>(pub &'a [u8]);

impl fmt::Debug for Bytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        for (i, &b) in self.0.iter().enumerate() {
            match b {
                b'\\' => f.write_str("\\"),
                b'"' => f.write_str("\\\""),
                b' '..=b'~' => f.write_char(b as char),
                b'\t' => f.write_str("\\t"),
                b'\n' => f.write_str("\\n"),
                b'\r' => f.write_str("\\r"),
                0 if !matches!(self.0.get(i + 1), Some(b'0'..=b'9')) => f.write_str("\\0"),
                _ => write!(f, "\\{b:03o}"),
            }?;
        }
        f.write_char('"')
    }
}

pub struct BlockLen(pub usize);

impl fmt::Debug for BlockLen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0;
        if len % 512 == 0 && len > 512 {
            write!(f, "{len} ({} * 512)", len / 512)
        } else {
            write!(f, "{len}")
        }
    }
}

impl fmt::Display for BlockLen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
