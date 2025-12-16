use crate::directory::DirectoryParser;
use crate::header::{Header, MagicString};
use crate::index::parse_tokens;
use crate::lumps::LumpCollection;
use crate::tokenizer::tokenize_lumps;
use std::sync::Arc;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

pub struct WadIndex {
    name: String,
    file_type: MagicString,
    size: usize,
    lumps: LumpCollection,
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
        let lumps = parse_tokens(tokens)?;
        let wad_reader = WadIndex {
            name,
            file_type,
            size,
            lumps,
        };

        Ok(wad_reader)
    }
    pub fn get_lumps(&self) -> &LumpCollection {
        &self.lumps
    }

    pub fn get_maps(&self) -> Option<&LumpCollection> {
        self.lumps.get_collection("MAPS")
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
        assert_eq!(wad.size, wad_bytes.len());
        assert_eq!(wad.file_type, MagicString::IWAD);
    }

    #[test]
    fn wad_can_index_lumps_from_doom1() {
        let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let wad =
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        assert!(!wad.get_lumps().is_empty());

        let num_levels = wad.get_maps().unwrap().collection_iter().len();
        assert_eq!(num_levels, 36);
    }

    #[test]
    fn wad_can_index_lumps_from_doom2() {
        let wad_data = include_bytes!("../assets/wad/freedoom2.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let wad =
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        assert!(!wad.get_lumps().is_empty());
        let num_levels = wad.get_maps().unwrap().collection_iter().len();
        assert_eq!(num_levels, 32);
    }
}
