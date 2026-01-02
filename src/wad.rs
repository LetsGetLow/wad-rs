use crate::audio::SoundSample;
use crate::header::{Header, MagicString};
use crate::index::{LumpNode, index_tokens};
use crate::map::MapIterator;
use crate::tokenizer::TokenIterator;
use std::collections::HashMap;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub struct WadIndex<'a> {
    header: Header,
    name: String,
    data: &'a [u8],
    file_type: MagicString,
    lump_index: HashMap<&'a str, LumpNode<'a>>,
}

impl<'a> WadIndex<'a> {
    pub fn from_bytes(name: String, data: &'a [u8]) -> Result<Self> {
        let size = data.len();
        if size < 12 {
            return Err("Data too small to contain valid WAD header".into());
        }
        let header_bytes: &[u8; 12] = data[0..12].try_into()?;
        let header = Header::try_from(header_bytes).map_err(|e| e.to_string())?;
        let file_type = header.identification;
        let lump_index = index_tokens(TokenIterator::new(header, &data)?)?;

        let wad_index = WadIndex {
            header,
            name,
            file_type,
            lump_index,
            data,
        };

        Ok(wad_index)
    }
    pub fn get_lump_index(&self) -> &HashMap<&'a str, LumpNode<'a>> {
        &self.lump_index
    }

    pub fn get_lump(&'_ self, namespaces: Vec<&str>, name: &str) -> Option<&LumpNode<'a>> {
        let mut current_index = &self.lump_index;
        for namespace in namespaces {
            if let Some(LumpNode::Namespace { children, .. }) =
                current_index.get(namespace)
            {
                current_index = children;
            } else {
                return None;
            }
        }
        current_index.get(name)
    }

    pub fn get_sound_sample(&self, name: &str) -> Result<Option<SoundSample>> {
        if let Some(lump_node) = self.lump_index.get(name) {
            if let LumpNode::Lump { lump, .. } = lump_node {
                let lump_data = lump.data();
                Ok(Some(SoundSample::try_from(lump_data)?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_file_type(&self) -> MagicString {
        self.file_type
    }

    // pub fn map_iter(&self) -> MapIterator<'_> {
    //     let tokens = TokenIterator::new(self.header, &self.data).unwrap();
    //     MapIterator::new(tokens)
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::header::MagicString;
//
//     #[test]
//     fn wad_can_be_created_from_bytes() {
//         let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
//         let wad_bytes = Rc::from(wad_data);
//
//         let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_bytes)).unwrap();
//
//         assert_eq!(wad.name, "freedoom1.wad");
//         assert_eq!(wad.file_type, MagicString::IWAD);
//     }
//
//     #[test]
//     fn wad_can_index_lumps_from_doom1() {
//         let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
//         let wad_bytes: Rc<[u8]> = Rc::from(wad_data);
//         let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_bytes)).unwrap();
//
//         assert!(!wad.get_lump_index().is_empty());
//     }
//
//     #[test]
//     fn wad_can_index_lumps_from_doom2() {
//         let wad_data = include_bytes!("../assets/wad/freedoom2.wad").to_vec();
//         let wad_bytes: Rc<[u8]> = Rc::from(wad_data);
//         let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_bytes)).unwrap();
//
//         assert!(!wad.get_lump_index().is_empty());
//     }
//
//     #[test]
//     fn wad_get_lump_by_namespaces() {
//         let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
//         let wad_bytes: Rc<[u8]> = Rc::from(wad_data);
//         let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_bytes)).unwrap();
//
//         let lump = wad.get_lump(vec!["P".to_string(), "P1".to_string()], "W13_A");
//         assert!(lump.is_some());
//     }
//
//     #[test]
//     fn wad_get_lump_by_namespaces_gives_none_on_invalid_lump_name() {
//         let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
//         let wad_bytes: Rc<[u8]> = Rc::from(wad_data);
//         let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_bytes)).unwrap();
//
//         let lump = wad.get_lump(
//             vec!["MAPS".to_string(), "E1M1".to_string()],
//             "NON_EXISTENT_LUMP",
//         );
//         assert!(lump.is_none());
//     }
//
//     #[test]
//     fn wad_get_lump_by_namespaces_gives_none_on_invalid_namespace() {
//         let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
//         let wad_bytes: Rc<[u8]> = Rc::from(wad_data);
//         let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_bytes)).unwrap();
//
//         let lump = wad.get_lump(
//             vec!["MAPS".to_string(), "NON_EXISTENT_NAMESPACE".to_string()],
//             "THINGS",
//         );
//         assert!(lump.is_none());
//     }
//
//     #[test]
//     fn wad_index_get_lumps_without_namespaces() {
//         let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
//         let wad_bytes: Rc<[u8]> = Rc::from(wad_data);
//         let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_bytes)).unwrap();
//
//         let lump = wad.get_lump(vec![], "DSPISTOL");
//         assert!(lump.is_some());
//     }
//
// #[test]
// fn wad_index_can_provide_a_map_iterator() {
//     let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
//     let wad_bytes: Rc<[u8]> = Rc::from(wad_data);
//     let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_bytes)).unwrap();
//     let map_iterator = wad.map_iter();
//     assert_eq!(map_iterator.count(), 36); // Freedoom1 has 36 maps
//
//     let mut map_iterator = wad.map_iter();
//     let first_map = map_iterator.next();
//     assert!(first_map.is_some());
//     let first_map = first_map.unwrap();
//     assert!(first_map.name().eq("E1M1"));
//     let last_map = map_iterator.last().unwrap();
//     assert_eq!(last_map.name().to_owned(), "E4M9".to_string());
// }
// }
