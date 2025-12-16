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
