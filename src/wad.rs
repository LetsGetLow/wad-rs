use crate::directory::{DirectoryParser, DirectoryRef};
use crate::header::{Header, MagicString};
use crate::tokenizer::tokenize_lumps;
use crate::{LumpToken, index_tokens};
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

pub struct WadIndex {
    name: String,
    file_type: MagicString,
    lump_index: HashMap<String, usize>,
    tokens: Vec<LumpToken>,
}

impl WadIndex {
    pub fn from_bytes(name: String, data: Arc<[u8]>) -> Result<Self> {
        let size = data.len();
        if size < 12 {
            return Err("Data too small to contain valid WAD header".into());
        }
        let header_bytes: &[u8; 12] = data[0..12].try_into()?;
        let header = Header::try_from(header_bytes).map_err(|e| e.to_string())?;
        let file_type = header.identification;
        let directory = DirectoryParser::new(Arc::clone(&data), header)?;
        let tokens = tokenize_lumps(directory.iter(), Arc::clone(&data));
        let lump_index = index_tokens(&tokens)?;
        let wad_reader = WadIndex {
            name,
            file_type,
            tokens,
            lump_index,
        };

        Ok(wad_reader)
    }
    pub fn get_lump_index(&self) -> &HashMap<String, usize> {
        &self.lump_index
    }

    pub fn get_lump_by_namespaces(
        &self,
        namespaces: Vec<String>,
        name: &str,
    ) -> Option<&DirectoryRef> {
        let namespace = namespaces
            .iter()
            .fold("".to_string(), |acc, namespase| acc.add(namespase).add("/"));

        let full_name = namespace.add(name);

        let index = *self.lump_index.get(&full_name)?;
        if let LumpToken::Lump(_, dir_ref) = &self.tokens[index] {
            Some(dir_ref)
        } else {
            None
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_file_type(&self) -> MagicString {
        self.file_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::MagicString;
    use std::sync::Arc;

    #[test]
    fn wad_can_be_created_from_bytes() {
        let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);

        let wad =
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        assert_eq!(wad.name, "freedoom1.wad");
        assert_eq!(wad.file_type, MagicString::IWAD);
    }

    #[test]
    fn wad_can_index_lumps_from_doom1() {
        let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let wad =
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        assert!(!wad.get_lump_index().is_empty());
    }

    #[test]
    fn wad_can_index_lumps_from_doom2() {
        let wad_data = include_bytes!("../assets/wad/freedoom2.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let wad =
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        assert!(!wad.get_lump_index().is_empty());
    }

    #[test]
    fn wad_get_lump_by_namespaces() {
        let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let wad =
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        let lump =
            wad.get_lump_by_namespaces(vec!["P".to_string(), "P1".to_string()], "W13_A");
        assert!(lump.is_some());
    }

    #[test]
    fn wad_get_lump_by_namespaces_gives_none_on_invalid_lump_name() {
        let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let wad =
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        let lump = wad.get_lump_by_namespaces(
            vec!["MAPS".to_string(), "E1M1".to_string()],
            "NON_EXISTENT_LUMP",
        );
        assert!(lump.is_none());
    }

    #[test]
    fn wad_get_lump_by_namespaces_gives_none_on_invalid_namespace() {
        let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let wad =
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        let lump = wad.get_lump_by_namespaces(
            vec!["MAPS".to_string(), "NON_EXISTENT_NAMESPACE".to_string()],
            "THINGS",
        );
        assert!(lump.is_none());
    }
}
