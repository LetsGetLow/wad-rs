use crate::header::Header;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const DIRECTORY_ENTRY_SIZE: usize = 16;
const DIRECTORY_NAME_LENGTH: usize = 8;

pub fn hash_from_name(name: &str) -> u64 {
    let mut name_bytes = [0u8; DIRECTORY_NAME_LENGTH];
    let bytes = name.as_bytes();
    let len = bytes.len().min(DIRECTORY_NAME_LENGTH);
    name_bytes[..len].copy_from_slice(&bytes[..len]);
    u64::from_le_bytes(name_bytes)
}

pub fn name_from_hash(hash: u64) -> String {
    let name_bytes = hash.to_le_bytes();
    let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(DIRECTORY_NAME_LENGTH);
    let name_slice = &name_bytes[..end];
    String::from_utf8_lossy(name_slice).to_ascii_uppercase().to_string()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectoryRef {
    start: usize,
    end: usize,
    name_offset: usize,
}

impl DirectoryRef {
    pub fn new(start: usize, end: usize, name_offset: usize) -> Self {
        Self {
            start,
            end,
            name_offset,
        }
    }

    pub fn name_offset(&self) -> usize {
        self.name_offset
    }

    pub fn name(&self, data: Arc<[u8]>) -> String {
        let slice = &data[self.name_offset..self.name_offset + DIRECTORY_NAME_LENGTH];
        let end = slice.iter().position(|&b| b == 0).unwrap_or(DIRECTORY_NAME_LENGTH);
        let name_bytes = &slice[..end];
        String::from_utf8_lossy(name_bytes).to_ascii_uppercase().to_string()
    }

    pub fn range(&self) -> (usize, usize) {
        (self.start, self.end)
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn is_marker(&self) -> bool {
        self.start == self.end
    }

    pub fn content(&self, data: Arc<[u8]>) -> Arc<[u8]> {
        Arc::from(&data[self.start..self.end])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectoryParser {
    start: usize,
    end: usize,
    current_index: usize,
    data: Arc<[u8]>,
}

impl DirectoryParser {
    pub fn new(data: Arc<[u8]>, header: Header) -> Result<Self> {
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
            data: Arc::clone(&self.data),
        }
    }
}

impl IntoIterator for DirectoryParser {
    type Item = DirectoryRef;
    type IntoIter = DirectoryIterator;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryIterator {
    current: usize,
    end: usize,
    data: Arc<[u8]>,
}

impl DirectoryIterator {
    pub(crate) fn seed_test_data(data: Arc<[u8]>, start: usize, end: usize) -> Self {
        Self {
            current: start,
            end,
            data,
        }
    }
}

impl Iterator for DirectoryIterator {
    type Item = DirectoryRef;

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
        Some(DirectoryRef::new(
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
    fn directory_ref_can_determine_name_by_data() {
        let data: Arc<[u8]> = Arc::from([b'T', b'E', b'S', b'T', 0, 0, 0, 0]);
        let dir_ref = DirectoryRef::new(0, 0, 0);
        assert_eq!(dir_ref.name(data), "TEST".to_ascii_uppercase());
    }

    #[test]
    fn directory_ref_can_store_start() {
        let r = DirectoryRef::new(0x1234, 0, 0);
        assert_eq!(r.start(), 0x1234);
    }

    #[test]
    fn directory_ref_can_store_end() {
        let r = DirectoryRef::new(0, 0x5678, 0);
        assert_eq!(r.end(), 0x5678);
    }

    #[test]
    fn directory_ref_can_store_name_offset() {
        let r = DirectoryRef::new(0, 0, 0x9ABC);
        assert_eq!(r.name_offset(), 0x9ABC);
    }

    #[test]
    fn directory_ref_can_store_range() {
        let r = DirectoryRef::new(0x1111, 0x2222, 0);
        assert_eq!(r.range(), (0x1111, 0x2222));
    }

    #[test]
    fn directory_ref_can_identify_marker() {
        let marker_ref = DirectoryRef::new(0x1000, 0x1000, 0);
        let non_marker_ref = DirectoryRef::new(0x1000, 0x2000, 0);
        assert!(marker_ref.is_marker());
        assert!(!non_marker_ref.is_marker());
    }

    #[test]
    fn directory_ref_can_extract_content_from_data() {
        let data: Arc<[u8]> = Arc::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let dir_ref = DirectoryRef::new(2, 7, 0);
        let content = dir_ref.content(Arc::clone(&data));
        assert_eq!(&*content, &[2, 3, 4, 5, 6]);
    }

    #[test]
    fn directory_parser_can_create_from_data_and_header() {
        let data: Arc<[u8]> = Arc::from([0; 64]);
        let header = Header {
            identification: crate::header::MagicString::IWAD,
            num_lumps: 2,
            info_table_offset: 16,
        };
        let parser = DirectoryParser::new(Arc::clone(&data), header).unwrap();
        assert_eq!(parser.start, 16);
        assert_eq!(parser.end, 16 + (2 * DIRECTORY_ENTRY_SIZE));
        assert_eq!(parser.data, data);
    }

    #[test]
    fn directory_parser_fails_with_insufficient_data() {
        let data: Arc<[u8]> = Arc::from([0; 16]);
        let header = Header {
            identification: crate::header::MagicString::IWAD,
            num_lumps: 2,
            info_table_offset: 16,
        };
        let result = DirectoryParser::new(Arc::clone(&data), header);
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

        let arc_data: Arc<[u8]> = Arc::from(data);
        let header = Header {
            identification: crate::header::MagicString::IWAD,
            num_lumps: 2,
            info_table_offset: 0,
        };
        let mut directory = DirectoryParser::new(Arc::clone(&arc_data), header).unwrap().iter();
        let first_ref = directory.next().unwrap();
        assert_eq!(first_ref.start(), 0x34);
        assert_eq!(first_ref.end(), 0x34 + 0x78);
        assert_eq!(first_ref.name(Arc::clone(&arc_data)), "ENTRYONE");
        let second_ref = directory.next().unwrap();
        assert_eq!(second_ref.start(), 0x9A);
        assert_eq!(second_ref.end(), 0x9A + 0xBC);
        assert_eq!(second_ref.name(arc_data), "ENTRYTWO");
        assert!(directory.next().is_none());
    }

    #[test]
    fn directory_parser_implements_into_iterator() {
        let mut data = Vec::with_capacity(DIRECTORY_ENTRY_SIZE);
        // Single ref
        data.extend(&0x00000010u32.to_le_bytes()); // pos
        data.extend(&0x00000020u32.to_le_bytes()); // size
        data.extend(b"SINGLEEN"); // name

        let arc_data: Arc<[u8]> = Arc::from(data);
        let header = Header {
            identification: crate::header::MagicString::IWAD,
            num_lumps: 1,
            info_table_offset: 0,
        };
        let directory_parser = DirectoryParser::new(Arc::clone(&arc_data), header).unwrap();
        let mut iter = directory_parser.into_iter();
        let dir_ref = iter.next().unwrap();
        assert_eq!(dir_ref.start(), 0x10);
        assert_eq!(dir_ref.end(), 0x10 + 0x20);
        assert_eq!(dir_ref.name(Arc::clone(&arc_data)), "SINGLEEN");
        assert!(iter.next().is_none());
    }
}
