use crate::header::Header;
use crate::lump::{LUMP_ENTRY_LENGTH, LumpRef};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum LumpToken<'a> {
    MarkerStart(&'a str),
    MarkerEnd(&'a str),
    MapMarker(&'a str),
    Lump(&'a str, LumpRef<'a>),
}

impl LumpToken<'_> {
    pub fn is_start_marker(name: &str) -> bool {
        name.ends_with("_START")
    }

    pub fn is_end_marker(name: &str) -> bool {
        name.ends_with("_END")
    }
}

fn is_map_marker(name: &str) -> bool {
    match name.as_bytes() {
        [b'M', b'A', b'P', d1, d2] => d1.is_ascii_digit() && d2.is_ascii_digit(),
        [b'E', d1, b'M', d2] => d1.is_ascii_digit() && d2.is_ascii_digit(),
        _ => false,
    }
}

pub struct TokenIterator<'a> {
    data: &'a [u8],
    directory_offset: usize,
    directory_end: usize,
}

impl<'a> TokenIterator<'a> {
    pub fn new(header: Header, data: &'a [u8]) -> Result<Self> {
        let directory_offset = header.info_table_offset as usize;
        let directory_end = directory_offset + (header.num_lumps as usize * LUMP_ENTRY_LENGTH);
        if data.len() < directory_end {
            Err("Data too small to contain directory entries".into())
        } else {
            Ok(TokenIterator {
                data,
                directory_offset,
                directory_end,
            })
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<LumpToken<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.directory_offset >= self.directory_end {
            return None;
        }

        let entry_offset = self.directory_offset;
        self.directory_offset += LUMP_ENTRY_LENGTH;

        // Safety: We are reading exactly 8 bytes from a valid slice of data we checked above
        // the overall data length is at least directory_end
        let (pos_bytes, len_bytes, name) = unsafe {
            let pos_ptr = self.data.as_ptr().add(entry_offset);
            let len_ptr = pos_ptr.add(4);
            let name_ptr = len_ptr.add(4);
            let name_bytes: &[u8; 8] = &*(name_ptr as *const [u8; 8]);
            let pos_bytes: &[u8; 4] = &*(pos_ptr as *const [u8; 4]);
            let len_bytes: &[u8; 4] = &*(len_ptr as *const [u8; 4]);

            (
                pos_bytes,
                len_bytes,
                std::str::from_utf8_unchecked(name_bytes).trim_end_matches('\0'),
            )
        };
        let pos = i32::from_le_bytes(*pos_bytes) as usize;
        let len = i32::from_le_bytes(*len_bytes) as usize;
        let data = &self.data[pos..pos + len];

        let lump_ref = LumpRef::new(data, name);

        let name = lump_ref.name();
        if len == 0 { // Marker lump
            if is_map_marker(&name) {
                Some(Ok(LumpToken::MapMarker(name)))
            } else if LumpToken::is_start_marker(&name) {
                Some(Ok(LumpToken::MarkerStart(name)))
            } else if LumpToken::is_end_marker(&name) {
                Some(Ok(LumpToken::MarkerEnd(name)))
            } else {
                Some(Err("Unknown marker type".into()))
            }
        } else {
            Some(Ok(LumpToken::Lump(name, lump_ref)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::MagicString;
    use std::rc::Rc;

    #[test]
    fn tokenize_lumps_produces_correct_start_end_marker_tokens() {
        let header = Header {
            identification: MagicString::try_from(b"IWAD").unwrap(),
            num_lumps: 2,
            info_table_offset: 0,
        };

        let data = vec![
            // _START marker
            0, 0, 0, 0, 0, 0, 0, 0, b'_', b'S', b'T', b'A', b'R', b'T', 0, 0, // _END marker
            0, 0, 0, 0, 0, 0, 0, 0, b'_', b'E', b'N', b'D', 0, 0, 0, 0,
        ];

        let mut tokens = TokenIterator::new(header, &data).unwrap();

        let first_token = tokens.next().unwrap().unwrap();
        assert_eq!(first_token, LumpToken::MarkerStart("_START"));

        let second_token = tokens.next().unwrap().unwrap();
        assert_eq!(second_token, LumpToken::MarkerEnd("_END"));

        assert!(tokens.next().is_none());
    }

    #[test]
    fn tokenize_lumps_produces_map_marker_tokens() {
        let header = Header {
            identification: MagicString::try_from(b"IWAD").unwrap(),
            num_lumps: 2,
            info_table_offset: 0,
        };
        let data = Rc::from(vec![
            // MAP01 marker Doom2, Heretic style
            0, 0, 0, 0, 0, 0, 0, 0, b'M', b'A', b'P', b'0', b'1', 0, 0, 0,
            // E1M2 marker Doom style
            0, 0, 0, 0, 0, 0, 0, 0, b'E', b'1', b'M', b'2', 0, 0, 0, 0,
        ]);
        let mut tokens = TokenIterator::new(header, &data).unwrap();

        let first_token = tokens.next().unwrap().unwrap();
        match first_token {
            LumpToken::MapMarker(name) => assert_eq!(name, "MAP01"),
            _ => panic!("Expected MapMarker token for MAP01"),
        }

        let second_token = tokens.next().unwrap().unwrap();
        match second_token {
            LumpToken::MapMarker(name) => assert_eq!(name, "E1M2"),
            _ => panic!("Expected MapMarker token for E1M2"),
        }
    }
}
