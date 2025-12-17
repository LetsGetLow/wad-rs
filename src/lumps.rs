use std::sync::Arc;

const LUMP_NAME_LENGTH: usize = 8;

pub fn is_map_lump(name: &String) -> bool {
    matches!(
        name.as_str(),
        "THINGS"
            | "LINEDEFS"
            | "SIDEDEFS"
            | "VERTEXES"
            | "SECTORS"
            | "SSECTORS"
            | "SEGS"
            | "NODES"
            | "REJECT"
            | "BLOCKMAP"
            | "BEHAVIOR"
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LumpRef {
    start: usize,
    end: usize,
    name_offset: usize,
}

impl LumpRef {
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
        let slice = &data[self.name_offset..self.name_offset + LUMP_NAME_LENGTH];
        let end = slice.iter().position(|&b| b == 0).unwrap_or(LUMP_NAME_LENGTH);
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

    pub fn extract_content(&self, data: Arc<[u8]>) -> Arc<[u8]> {
        Arc::from(&data[self.start..self.end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_map_lump_identifies_map_lumps() {
        let map_lumps = vec![
            "THINGS",
            "LINEDEFS",
            "SIDEDEFS",
            "VERTEXES",
            "SECTORS",
            "SSECTORS",
            "SEGS",
            "NODES",
            "REJECT",
            "BLOCKMAP",
            "BEHAVIOR",
        ];

        for lump in map_lumps {
            assert!(is_map_lump(&lump.to_string()));
        }

        let non_map_lumps = vec!["TEXTURE1", "FLAT1", "SOUND1", "GRAPHICS", "LEVEL1"];
        for lump in non_map_lumps {
            assert!(!is_map_lump(&lump.to_string()));
        }
    }

    #[test]
    fn directory_ref_can_determine_name_by_data() {
        let data: Arc<[u8]> = Arc::from([b'T', b'E', b'S', b'T', 0, 0, 0, 0]);
        let dir_ref = LumpRef::new(0, 0, 0);
        assert_eq!(dir_ref.name(data), "TEST".to_ascii_uppercase());
    }

    #[test]
    fn directory_ref_can_store_start() {
        let r = LumpRef::new(0x1234, 0, 0);
        assert_eq!(r.start(), 0x1234);
    }

    #[test]
    fn directory_ref_can_store_end() {
        let r = LumpRef::new(0, 0x5678, 0);
        assert_eq!(r.end(), 0x5678);
    }

    #[test]
    fn directory_ref_can_store_name_offset() {
        let r = LumpRef::new(0, 0, 0x9ABC);
        assert_eq!(r.name_offset(), 0x9ABC);
    }

    #[test]
    fn directory_ref_can_store_range() {
        let r = LumpRef::new(0x1111, 0x2222, 0);
        assert_eq!(r.range(), (0x1111, 0x2222));
    }

    #[test]
    fn directory_ref_can_identify_marker() {
        let marker_ref = LumpRef::new(0x1000, 0x1000, 0);
        let non_marker_ref = LumpRef::new(0x1000, 0x2000, 0);
        assert!(marker_ref.is_marker());
        assert!(!non_marker_ref.is_marker());
    }

    #[test]
    fn directory_ref_can_extract_content_from_data() {
        let data: Arc<[u8]> = Arc::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let dir_ref = LumpRef::new(2, 7, 0);
        let content = dir_ref.extract_content(Arc::clone(&data));
        assert_eq!(&*content, &[2, 3, 4, 5, 6]);
    }
}