use std::mem;

use crate::util::U16Le;

/// Detects whether the file data is ASCII text.
pub fn is_text(data: &[u8]) -> bool {
    data.iter()
        .all(|&b| matches!(b, 0x07..=0x0f | 0x1b | b' '..=b'~'))
}

/// a.out header.
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct AOut {
    /// Magic number.
    pub magic: U16Le,
    /// Size of the text segment.
    pub text_size: U16Le,
    /// Size of the data segment.
    pub data_size: U16Le,
    /// Size of the bss segment.
    pub bss_size: U16Le,
    /// Size of the symbol table.
    pub symtab_size: U16Le,
    /// Entry point.
    pub entry_point: U16Le,
    /// Unused.
    pub unused: U16Le,
    /// Whether relocation info is stripped.
    pub flag: U16Le,
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

impl AOut {
    pub fn parse(data: &[u8]) -> Option<&Self> {
        // TODO: Handle V1Normal programs shorter than 8 words.
        let aout: &AOut = unsafe { mem::transmute(data.first_chunk::<16>()?) };
        if Magic::from_first(aout.magic.get()).is_some_and(Magic::is_aout) {
            Some(aout)
        } else {
            None
        }
    }

    pub fn magic(&self) -> Magic {
        Magic::from_first(self.magic.get()).unwrap()
    }

    pub fn file_size(&self) -> Option<usize> {
        // TODO: Consult documentation for the headers.
        match self.magic() {
            Magic::AnyNormal => Some(
                16 + self.text_size.get() as usize
                    + self.data_size.get() as usize
                    + self.symtab_size.get() as usize,
            ),
            Magic::V1Normal => Some(
                self.text_size.get() as usize
                    + self.data_size.get() as usize
                    + self.bss_size.get() as usize,
            ),
            Magic::AnyRoText
            | Magic::AnySplitID
            | Magic::BsdOverlay
            | Magic::BsdROverlay
            | Magic::V1Raw => unimplemented!(),
            Magic::Algol68 | Magic::Shell => None,
        }
    }
}

impl Magic {
    /// Detects a magic number for the file data.
    pub fn detect(data: &[u8]) -> Option<Self> {
        let first_word = U16Le(*data.first_chunk()?).get();
        let magic = Magic::from_first(first_word)?;
        // Only handle secondary magic numbers for Algol 68, because its primary
        // magic number is so generic. The Unix version detection logic of Apout
        // is not necessary.
        match magic {
            Magic::Algol68 => {
                if data.len() >= 6
                    && u16::from_le_bytes(*data[4..].first_chunk().unwrap()) == 0o0107116
                {
                    Some(Magic::Algol68)
                } else {
                    None
                }
            }
            _ => Some(magic),
        }
    }

    pub fn from_first(first_word: u16) -> Option<Self> {
        #![allow(non_upper_case_globals)]
        macro_rules! match_magic(($magic:expr, $($variant:ident)|*) => {{
            $(const $variant: u16 = Magic::$variant as _;)*
            match $magic {
                $($variant => Some(Magic::$variant),)*
                _ => None
            }
        }});
        match_magic!(
            first_word,
            V1Normal
                | AnyNormal
                | AnyRoText
                | AnySplitID
                | BsdOverlay
                | BsdROverlay
                | V1Raw
                | Algol68
                | Shell
        )
    }

    pub fn is_aout(self) -> bool {
        matches!(
            self,
            Magic::V1Normal
                | Magic::AnyNormal
                | Magic::AnyRoText
                | Magic::AnySplitID
                | Magic::BsdOverlay
                | Magic::BsdROverlay
        )
    }
}
