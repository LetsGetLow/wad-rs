use crate::error::WadError;
use crate::header::Header;
use crate::lump::{LUMP_ENTRY_LENGTH, LumpRef};

type Error = WadError;
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
    pub fn is_map_marker(name: &str) -> bool {
        match name.as_bytes() {
            [b'M', b'A', b'P', d1, d2] => d1.is_ascii_digit() && d2.is_ascii_digit(),
            [b'E', d1, b'M', d2] => d1.is_ascii_digit() && d2.is_ascii_digit(),
            _ => false,
        }
    }
}

impl<'a> TryFrom<LumpRef<'a>> for LumpToken<'a> {
    type Error = Error;

    fn try_from(lump_ref: LumpRef<'a>) -> Result<Self> {
        let name = lump_ref.name();
        if lump_ref.is_marker() {
            if LumpToken::is_map_marker(name) {
                Ok(LumpToken::MapMarker(name))
            } else if LumpToken::is_start_marker(name) {
                Ok(LumpToken::MarkerStart(name))
            } else if LumpToken::is_end_marker(name) {
                Ok(LumpToken::MarkerEnd(name))
            } else {
                Err(WadError::UnknownMarkerType)
            }
        } else {
            Ok(LumpToken::Lump(name, lump_ref))
        }
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
            Err(WadError::TokenDataTooSmall)
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
        // TODO: Even though we checked bounds in new(), it could make sense to double-check here
        if self.directory_offset >= self.directory_end {
            return None;
        }

        let entry_offset = self.directory_offset;
        self.directory_offset += LUMP_ENTRY_LENGTH;


        let name_offset = entry_offset + 8;
        let name_bytes = &self.data.get(name_offset..name_offset + 8)?; // should always work
        // Safety: We are reading exactly 8 bytes from a valid slice of data we checked in new()
        // the overall data length is at least directory_end
        let name = unsafe { std::str::from_utf8_unchecked(name_bytes) }.trim_end_matches('\0');

        let pos_bytes: [u8; 4] = self
            .data
            .get(entry_offset..entry_offset + 4)?
            .try_into()
            .ok()?; // should always work
        let pos = i32::from_le_bytes(pos_bytes) as usize;

        let len_bytes: [u8; 4] = self
            .data
            .get(entry_offset + 4..entry_offset + 8)?
            .try_into()
            .ok()?; // should always work
        let len = i32::from_le_bytes(len_bytes) as usize;

        let data = &self.data[pos..pos + len];

        Some(LumpRef::new(name, data).try_into())
    }
}

// Helper function to create fake token data for testing
#[cfg(test)]
pub fn fake_token_data<'a>(entries: &[(&'a str, &'a [u8])]) -> (Header, Vec<u8>) {
    use crate::header::{Header, MagicString};
    use crate::lump::LUMP_ENTRY_LENGTH;

    let dir_size = entries.len() * LUMP_ENTRY_LENGTH;

    let mut dir = Vec::with_capacity(dir_size);
    let mut lump_data = Vec::new();

    for (name, payload) in entries {
        // pos muss relativ zum Start des gesamten Blobs sein: [dir][lump_data]
        let pos = (dir_size + lump_data.len()) as i32;
        let len = payload.len() as i32;

        lump_data.extend_from_slice(payload);

        dir.extend_from_slice(&pos.to_le_bytes());
        dir.extend_from_slice(&len.to_le_bytes());

        let mut name_buf = [0u8; 8];
        name_buf[..name.len()].copy_from_slice(name.as_bytes());
        dir.extend_from_slice(&name_buf);
    }

    dir.extend_from_slice(&lump_data);

    (
        Header {
            identification: MagicString::try_from(b"IWAD").unwrap(),
            num_lumps: entries.len() as i32,
            info_table_offset: 0,
        },
        dir,
    )
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
