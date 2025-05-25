use std::fmt::{self, Write};

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
