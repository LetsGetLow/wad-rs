use crate::header::Header;
use crate::lump::LumpRef;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const DIRECTORY_ENTRY_SIZE: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct DirectoryParser {
    start: usize,
    end: usize,
    current_index: usize,
    data: Rc<[u8]>,
}

impl DirectoryParser {
    pub fn new(data: Rc<[u8]>, header: Header) -> Result<Self> {
        let start = header.info_table_offset as usize;
        let end = start + (header.num_lumps as usize * DIRECTORY_ENTRY_SIZE);
        if end > data.len() {
            return Err("Data too small to contain directory entries".into());
        }

        Ok(DirectoryParser {
            data,
            start,
            end,
            current_index: start,
        })
    }

    pub fn iter(&self) -> DirectoryIterator {
        DirectoryIterator {
            current: self.start,
            end: self.end,
            data: Rc::clone(&self.data),
        }
    }
}

impl IntoIterator for DirectoryParser {
    type Item = LumpRef;
    type IntoIter = DirectoryIterator;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryIterator {
    current: usize,
    end: usize,
    data: Rc<[u8]>,
}

impl DirectoryIterator {
    /// Seeds test data for the iterator not for production use
    pub(crate) fn seed_test_data(data: Rc<[u8]>, start: usize, end: usize) -> Self {
        Self {
            current: start,
            end,
            data,
        }
    }
}

impl Iterator for DirectoryIterator {
    type Item = LumpRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let start = self.current;
        let end = self.current + DIRECTORY_ENTRY_SIZE;
        let entry_bytes: &[u8; 16] = self.data[start..end].try_into().unwrap();
        self.current = end;
        let pos = i32::from_le_bytes([entry_bytes[0], entry_bytes[1], entry_bytes[2], entry_bytes[3]]) as usize;
        let len = i32::from_le_bytes([entry_bytes[4], entry_bytes[5], entry_bytes[6], entry_bytes[7]]) as usize;
        Some(LumpRef::new(
            pos,
            pos + len,
            start + 8,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn directory_parser_can_create_from_data_and_header() {
        let data = Rc::from([0; 64]);
        let header = Header {
            identification: crate::header::MagicString::IWAD,
            num_lumps: 2,
            info_table_offset: 16,
        };
        let parser = DirectoryParser::new(Rc::clone(&data), header).unwrap();
        assert_eq!(parser.start, 16);
        assert_eq!(parser.end, 16 + (2 * DIRECTORY_ENTRY_SIZE));
        assert_eq!(parser.data, data);
    }

    #[test]
    fn directory_parser_fails_with_insufficient_data() {
        let data  = Rc::from([0; 16]);
        let header = Header {
            identification: crate::header::MagicString::IWAD,
            num_lumps: 2,
            info_table_offset: 16,
        };
        let result = DirectoryParser::new(Rc::clone(&data), header);
        assert!(result.is_err());
    }

    #[test]
    fn directory_parser_can_iterate_entries() {
        let mut data = Vec::with_capacity(DIRECTORY_ENTRY_SIZE * 2);
        // First ref
        data.extend(&0x00000034i32.to_le_bytes()); // pos
        data.extend(&0x00000078i32.to_le_bytes()); // size
        data.extend(b"ENTRYONE"); // name
        // Second ref
        data.extend(&0x0000009Ai32.to_le_bytes()); // pos
        data.extend(&0x000000BCi32.to_le_bytes()); // size
        data.extend(b"ENTRYTWO"); // name

        let data = Rc::from(data);
        let header = Header {
            identification: crate::header::MagicString::IWAD,
            num_lumps: 2,
            info_table_offset: 0,
        };
        let mut directory = DirectoryParser::new(Rc::clone(&data), header).unwrap().iter();
        let first_ref = directory.next().unwrap();
        assert_eq!(first_ref.start(), 0x34);
        assert_eq!(first_ref.end(), 0x34 + 0x78);
        assert_eq!(unsafe {first_ref.name(&data)}, "ENTRYONE");
        let second_ref = directory.next().unwrap();
        assert_eq!(second_ref.start(), 0x9A);
        assert_eq!(second_ref.end(), 0x9A + 0xBC);
        unsafe { assert_eq!(second_ref.name(&data), "ENTRYTWO"); }
        assert!(directory.next().is_none());
    }

    #[test]
    fn directory_parser_implements_into_iterator() {
        let mut data = Vec::with_capacity(DIRECTORY_ENTRY_SIZE);
        // Single ref
        data.extend(&0x00000010u32.to_le_bytes()); // pos
        data.extend(&0x00000020u32.to_le_bytes()); // size
        data.extend(b"SINGLEEN"); // name

        let data: Rc<[u8]> = Rc::from(data);
        let header = Header {
            identification: crate::header::MagicString::IWAD,
            num_lumps: 1,
            info_table_offset: 0,
        };
        let directory_parser = DirectoryParser::new(Rc::clone(&data), header).unwrap();
        let mut iter = directory_parser.into_iter();
        let dir_ref = iter.next().unwrap();
        assert_eq!(dir_ref.start(), 0x10);
        assert_eq!(dir_ref.end(), 0x10 + 0x20);
        unsafe { assert_eq!(dir_ref.name(&data), "SINGLEEN"); }
        assert!(iter.next().is_none());
    }
}
