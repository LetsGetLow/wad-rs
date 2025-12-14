use crate::header::Header;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const DIRECTORY_ENTRY_SIZE: usize = 16;
const DIRECTORY_NAME_LENGTH: usize = 8;

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directory {
    start: usize,
    end: usize,
    current_index: usize,
    data: Arc<[u8]>,
}

impl Directory {
    pub fn new(data: Arc<[u8]>, header: Header) -> Result<Self> {
        let start = header.info_table_offset as usize;
        let end = start + (header.num_lumps as usize * DIRECTORY_ENTRY_SIZE);
        if end > data.len() {
            return Err("Data too small to contain directory entries".into());
        }

        Ok(Directory {
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

impl IntoIterator for Directory {
    type Item = DirectoryRef;
    type IntoIter = DirectoryIterator;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct DirectoryIterator {
    current: usize,
    end: usize,
    data: Arc<[u8]>,
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
    fn directory_can_iterate_entries() {
        let mut data = Vec::with_capacity(DIRECTORY_ENTRY_SIZE * 2);
        // First ref
        data.extend(&0x00000034u32.to_le_bytes()); // pos
        data.extend(&0x00000078u32.to_le_bytes()); // size
        data.extend(b"ENTRYONE"); // name
        // Second ref
        data.extend(&0x0000009Au32.to_le_bytes()); // pos
        data.extend(&0x000000BCu32.to_le_bytes()); // size
        data.extend(b"ENTRYTWO"); // name

        let arc_data: Arc<[u8]> = Arc::from(data);
        let header = Header {
            identification: crate::header::HeaderId::IWAD,
            num_lumps: 2,
            info_table_offset: 0,
        };
        let mut directory = Directory::new(Arc::clone(&arc_data), header).unwrap().iter();
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
}
