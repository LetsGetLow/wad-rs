pub const LUMP_NAME_LENGTH: usize = 8;
pub const LUMP_ENTRY_LENGTH: usize = 16;

/// A refence to a lump data and it's name
/// this struct does not own any data, it just points to offsets in the WAD file
/// it should be used in conjunction with the WAD file data to extract the lump name and data
///
/// # Safety
/// This struct uses unsafe code to extract the lump name and data from the WAD file data
/// the caller must ensure that the provided data slice is valid and contains the lump data
///
/// # Fields
/// - `start`: The start offset of the lump data in the WAD file
/// - `end`: The end offset of the lump data in the WAD file
/// - `name_offset`: The offset of the lump name in the WAD file
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LumpRef<'a> {
    data: &'a [u8],
    name: &'a str,
}

impl<'a> LumpRef<'a> {
    /// Creates a new LumpRef
    pub fn new(data: &'a [u8], name: &'a str) -> Self {
        Self { data, name }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn is_marker(&self) -> bool {
        self.data.len() == 0
    }

    // Extracts the lump content from the provided data
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::rc::Rc;
//
//     #[test]
//     fn is_map_lump_identifies_map_lumps() {
//         let map_lumps = vec![
//             "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SECTORS", "SSECTORS", "SEGS", "NODES",
//             "REJECT", "BLOCKMAP", "BEHAVIOR",
//         ];
//
//         for lump in map_lumps {
//             assert!(is_map_lump(&lump.to_string()));
//         }
//
//         let non_map_lumps = vec!["TEXTURE1", "FLAT1", "SOUND1", "GRAPHICS", "LEVEL1"];
//         for lump in non_map_lumps {
//             assert!(!is_map_lump(&lump.to_string()));
//         }
//     }
//
//     #[test]
//     fn directory_ref_can_determine_name_by_data() {
//         let data = Rc::from([b'T', b'E', b'S', b'T', 0, 0, 0, 0]);
//         let dir_ref = LumpRef::new(0, 0, 0);
//         assert_eq!(dir_ref.name(&data), "TEST");
//     }
//
//     #[test]
//     fn directory_ref_can_store_start() {
//         let r = LumpRef::new(0x1234, 0, 0);
//         assert_eq!(r.start(), 0x1234);
//     }
//
//     #[test]
//     fn directory_ref_can_store_end() {
//         let r = LumpRef::new(0, 0x5678, 0);
//         assert_eq!(r.end(), 0x5678);
//     }
//
//     #[test]
//     fn directory_ref_can_store_name_offset() {
//         let r = LumpRef::new(0, 0, 0x9ABC);
//         assert_eq!(r.name_offset(), 0x9ABC);
//     }
//
//     #[test]
//     fn directory_ref_can_store_range() {
//         let r = LumpRef::new(0x1111, 0x2222, 0);
//         assert_eq!(r.range(), (0x1111, 0x2222));
//     }
//
//     #[test]
//     fn directory_ref_can_identify_marker() {
//         let marker_ref = LumpRef::new(0x1000, 0x1000, 0);
//         let non_marker_ref = LumpRef::new(0x1000, 0x2000, 0);
//         assert!(marker_ref.is_marker());
//         assert!(!non_marker_ref.is_marker());
//     }
//
//     #[test]
//     fn directory_ref_can_extract_content_from_data() {
//         let data = Rc::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
//         let dir_ref = LumpRef::new(2, 7, 0);
//         let content = dir_ref.extract_content(&data);
//         assert_eq!(&*content, &[2, 3, 4, 5, 6]);
//     }
// }
