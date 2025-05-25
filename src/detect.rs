/// Detects whether the file data is ASCII text.
pub fn is_text(data: &[u8]) -> bool {
    data.iter().all(|&b| matches!(b, 0x07..=0x0f | b' '..=b'~'))
}

/// Magic number for an a.out binary or a shell script.
///
/// Follows the logic of [Apout](https://github.com/DoctorWkt/Apout/blob/e88a446ace064f5a41e1a47d9ae8278b83b27a20/aout.c#L89).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Magic {
    // Normal: V1, six words long.
    V1Normal = 0o0405,
    // Normal: V5, V6, V7, 2.11BSD.
    AnyNormal = 0o0407,
    // Read-only text: V5, V6, V7, 2.11BSD.
    AnyRoText = 0o0410,
    // Separated I&D: V5, V6, V7, 2.11BSD.
    AnySplitID = 0o0411,
    // 2.11BSD overlay, non-separate..
    BsdOverlay = 0o0430,
    // 2.11BSD overlay, separate..
    BsdROverlay = 0o0431,
    // V1 'raw' binary: `rm`, `ln`, `chmod` from s2.
    V1Raw = 0o0104421,
    // Algol 68 binary.
    Algol68 = 0o0,
    // Shell script shebang, i.e., `#!`.
    Shell = const { u16::from_le_bytes(*b"#!") },
}

/// Detects a magic number for the file data.
pub fn detect_magic(data: &[u8]) -> Option<Magic> {
    #![allow(non_upper_case_globals)]
    macro_rules! match_magic(($magic:expr, $($variant:ident)|*) => {{
        $(const $variant: u16 = Magic::$variant as _;)*
        match $magic {
            $($variant => Some(Magic::$variant),)*
            _ => None
        }
    }});
    let magic = match_magic!(
        u16::from_le_bytes(*data.first_chunk()?),
        V1Normal
            | AnyNormal
            | AnyRoText
            | AnySplitID
            | BsdOverlay
            | BsdROverlay
            | V1Raw
            | Algol68
            | Shell
    )?;
    // Only handle secondary magic numbers for Algol 68, because its primary
    // magic number is so generic. The version detection logic of Apout is not
    // necessary.
    match magic {
        Magic::Algol68 => {
            if data.len() >= 6 && u16::from_le_bytes(*data[4..].first_chunk().unwrap()) == 0o0107116
            {
                Some(Magic::Algol68)
            } else {
                None
            }
        }
        _ => Some(magic),
    }
}
