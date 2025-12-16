use crate::directory::DirectoryRef;
use std::collections::hash_map::Iter;
use std::collections::{HashMap, hash_map};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct LumpCollection {
    lumps: HashMap<String, DirectoryRef>,
    collections: HashMap<String, LumpCollection>,
}

impl LumpCollection {
    pub fn add_lump(&mut self, name: &str, reference: DirectoryRef) -> Result<()> {
        self.lumps.insert(name.to_string(), reference);
        Ok(())
    }

    pub fn get_lump(&self, name: &str) -> Option<&DirectoryRef> {
        self.lumps.get(name)
    }

    pub fn add_collection(&mut self, name: &str, collection: LumpCollection) -> Result<()> {
        if self.collections.contains_key(name) {
            return Err(format!("Sub-collection with name '{}' already exists", name).into());
        }
        self.collections.insert(name.to_string(), collection);
        Ok(())
    }

    pub fn get_collection(&self, name: &str) -> Option<&LumpCollection> {
        self.collections.get(name)
    }

    pub fn is_empty(&self) -> bool {
        self.lumps.is_empty() && self.collections.is_empty()
    }
    
    pub fn has_lumps(&self) -> bool {
        !self.lumps.is_empty()
    }
    
    pub fn has_collections(&self) -> bool {
        !self.collections.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, String, DirectoryRef> {
        self.lumps.iter()
    }

    pub fn collection_iter(&self) -> hash_map::Iter<'_, String, LumpCollection> {
        self.collections.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapLump {
    things: Option<DirectoryRef>,
    line_defs: Option<DirectoryRef>,
    side_defs: Option<DirectoryRef>,
    vertices: Option<DirectoryRef>,
    sectors: Option<DirectoryRef>,
    sub_sectors: Option<DirectoryRef>,
    segs: Option<DirectoryRef>,
    nodes: Option<DirectoryRef>,
    reject: Option<DirectoryRef>,
    block_map: Option<DirectoryRef>,
}

impl MapLump {
    pub fn new() -> Self {
        Self {
            things: None,
            line_defs: None,
            side_defs: None,
            vertices: None,
            sectors: None,
            sub_sectors: None,
            segs: None,
            nodes: None,
            reject: None,
            block_map: None,
        }
    }

    pub fn is_map_lump(name: &str) -> bool {
        matches!(
            name,
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
        )
    }

    pub fn add_lump(&mut self, name: &str, dir_ref: DirectoryRef) -> bool {
        match name {
            "THINGS" => self.things = Some(dir_ref),
            "LINEDEFS" => self.line_defs = Some(dir_ref),
            "SIDEDEFS" => self.side_defs = Some(dir_ref),
            "VERTEXES" => self.vertices = Some(dir_ref),
            "SECTORS" => self.sectors = Some(dir_ref),
            "SSECTORS" => self.sub_sectors = Some(dir_ref),
            "SEGS" => self.segs = Some(dir_ref),
            "NODES" => self.nodes = Some(dir_ref),
            "REJECT" => self.reject = Some(dir_ref),
            "BLOCKMAP" => self.block_map = Some(dir_ref),
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lump_collection_add_and_get_lump() {
        let mut collection = LumpCollection::default();
        let dir_ref = DirectoryRef::new(0, 100, 8);
        collection.add_lump("TEST_LUMP", dir_ref.clone()).unwrap();

        let retrieved = collection.get_lump("TEST_LUMP").unwrap();
        assert_eq!(retrieved, &dir_ref);
    }

    #[test]
    fn add_and_get_sub_collection() {
        let mut parent_collection = LumpCollection::default();
        let mut sub_collection = LumpCollection::default();
        let dir_ref = DirectoryRef::new(0, 200, 8);
        sub_collection.add_lump("SUB_LUMP", dir_ref.clone()).unwrap();

        parent_collection
            .add_collection("SUB_COLLECTION", sub_collection.clone())
            .unwrap();

        let retrieved = parent_collection.get_collection("SUB_COLLECTION").unwrap();
        let sub_lump = retrieved.get_lump("SUB_LUMP").unwrap();
        assert_eq!(sub_lump, &dir_ref);
    }
}
