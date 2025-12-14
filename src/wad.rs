use crate::directory::Directory;
use crate::header::Header;
use crate::map::MapIndex;
use std::collections::HashMap;
use std::sync::Arc;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

pub struct WadReader {
    name: String,
    size: usize,
    header: Arc<Header>,
    directory: Arc<Directory>,
    data: Arc<[u8]>,
    pub maps: HashMap<String, MapIndex>,
}

impl WadReader {
    pub fn from_bytes(name: String, data: Arc<[u8]>) -> Result<Self> {
        let size = data.len();
        if size < 12 {
            return Err("Data too small to contain valid WAD header".into());
        }
        let header_bytes: &[u8; 12] = data[0..12].try_into()?;
        let header = Header::try_from(header_bytes).map_err(|e| e.to_string())?;
        let directory = Arc::new(Directory::new(Arc::clone(&data), header)?);
        let header = Arc::new(header);
        Ok(WadReader {
            name,
            size,
            header,
            directory,
            data,
            maps: HashMap::new(),
        })
    }

    pub fn index_lumps(&mut self) {
        let is_marker = |start: usize, end: usize| start == end;
        let is_map = |name: &String| {
            if name.len() < 4 {
                return false;
            }
            if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
                return false;
            }

            if name.starts_with("MAP") {
                return true;
            }

            let n = name.as_bytes();
            n[0] == b'E' && n[2] == b'M'
        };

        let mut maps = HashMap::new();
        let mut last_map_key: String = "".to_string();
        for dir_ref in self.directory.iter() {
            let name = dir_ref.name(Arc::clone(&self.data));
            if is_marker(dir_ref.start(), dir_ref.end()) && is_map(&name) {
                last_map_key = name.clone();
                maps.insert(name, MapIndex::new());
                continue;
            }

            if MapIndex::is_map_lump(&name) {
                if let Some(map_index) = maps.get_mut(&last_map_key) {
                    map_index.add_lump(&name, dir_ref);
                    continue;
                }
            }
        }

        self.maps = maps;
    }

    pub fn get_directory(&self) -> Arc<Directory> {
        Arc::clone(&self.directory)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_header(&self) -> &Header {
        &self.header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::HeaderId;
    use std::sync::Arc;

    #[test]
    fn wad_can_be_created_from_bytes() {
        let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);

        let wad =
            WadReader::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();

        assert_eq!(wad.name, "freedoom1.wad");
        assert_eq!(wad.size, wad_bytes.len());
        assert_eq!(wad.header.identification, HeaderId::IWAD);
        assert_eq!(
            wad.get_directory().iter().count(),
            wad.header.num_lumps as usize
        );
    }

    #[test]
    fn wad_can_index_lumps_from_doom1() {
        let wad_data = include_bytes!("../assets/wad/freedoom1.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let mut wad =
            WadReader::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();
        wad.index_lumps();
        assert!(!wad.maps.is_empty());
    }

    #[test]
    fn wad_can_index_lumps_from_doom2() {
        let wad_data = include_bytes!("../assets/wad/freedoom2.wad").to_vec();
        let wad_bytes: Arc<[u8]> = Arc::from(wad_data);
        let mut wad =
            WadReader::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_bytes)).unwrap();
        wad.index_lumps();
        assert!(!wad.maps.is_empty());
    }
}
