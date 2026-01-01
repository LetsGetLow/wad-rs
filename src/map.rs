use crate::lump::LumpRef;
use crate::tokenizer::{LumpToken, TokenIterator};

pub struct Map<'a> {
    name: &'a str,
    things: Option<LumpRef>,
    linedefs: Option<LumpRef>,
    sidedefs: Option<LumpRef>,
    vertexes: Option<LumpRef>,
    sectors: Option<LumpRef>,
    segs: Option<LumpRef>,
    ssectors: Option<LumpRef>,
    nodes: Option<LumpRef>,
    rejects: Option<LumpRef>,
    blockmap: Option<LumpRef>,
    behavior: Option<LumpRef>, // only for Heretic/Hexen
}

impl<'a> Map<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            things: None,
            linedefs: None,
            sidedefs: None,
            vertexes: None,
            sectors: None,
            segs: None,
            ssectors: None,
            nodes: None,
            rejects: None,
            blockmap: None,
            behavior: None, // only for Heretic/Hexen
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn things(&self) -> Option<&LumpRef> {
        self.things.as_ref()
    }

    pub fn linedefs(&self) -> Option<&LumpRef> {
        self.linedefs.as_ref()
    }

    pub fn sidedefs(&self) -> Option<&LumpRef> {
        self.sidedefs.as_ref()
    }

    pub fn vertexes(&self) -> Option<&LumpRef> {
        self.vertexes.as_ref()
    }

    pub fn sectors(&self) -> Option<&LumpRef> {
        self.sectors.as_ref()
    }

    pub fn segs(&self) -> Option<&LumpRef> {
        self.segs.as_ref()
    }

    pub fn ssectors(&self) -> Option<&LumpRef> {
        self.ssectors.as_ref()
    }

    pub fn nodes(&self) -> Option<&LumpRef> {
        self.nodes.as_ref()
    }

    pub fn rejects(&self) -> Option<&LumpRef> {
        self.rejects.as_ref()
    }

    pub fn blockmap(&self) -> Option<&LumpRef> {
        self.blockmap.as_ref()
    }
}

pub struct MapIterator<'a> {
    tokens: TokenIterator<'a>,
}

impl<'a> MapIterator<'a> {
    pub fn new(tokens: TokenIterator<'a>) -> Self {
        Self {
            tokens,
        }
    }
}

impl<'a> Iterator for MapIterator<'a> {
    type Item = Map<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.tokens.next()
            && let Ok(LumpToken::MapMarker(name)) = token
        {
            let mut map = Map::new(name);

            while let Some(result) = self.tokens.next() {
                if let Err(err) = result {
                    // TODO: implement error handling as needed
                    panic!("Map iterator returned an error: {}", err);
                }

                let token = result.unwrap();

                match token {
                    LumpToken::Lump(lump_name, lump_ref) => match lump_name {
                        "THINGS" => map.things = Some(lump_ref),
                        "LINEDEFS" => map.linedefs = Some(lump_ref),
                        "SIDEDEFS" => map.sidedefs = Some(lump_ref),
                        "VERTEXES" => map.vertexes = Some(lump_ref),
                        "SECTORS" => map.sectors = Some(lump_ref),
                        "SEGS" => map.segs = Some(lump_ref),
                        "SSECTORS" => map.ssectors = Some(lump_ref),
                        "NODES" => map.nodes = Some(lump_ref),
                        "REJECT" => map.rejects = Some(lump_ref),
                        "BLOCKMAP" => map.blockmap = Some(lump_ref),
                        "BEHAVIOR" => map.behavior = Some(lump_ref),
                        _ => break,
                    },
                    LumpToken::MapMarker(_)
                    | LumpToken::MarkerStart(_)
                    | LumpToken::MarkerEnd(_) => break,
                }
            }

            Some(map)
        } else {
            None
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::tokenizer::LumpToken;
//
//     #[test]
//     fn map_iterator_returns_none_for_empty_tokens() {
//         let tokens = Rc::new(vec![]);
//         let mut map_iterator = MapIterator::new(Rc::clone(&tokens));
//
//         assert!(map_iterator.next().is_none());
//     }
//
//     #[test]
//     fn map_iterator_returns_map_for_valid_tokens() {
//         let tokens = Rc::new(vec![
//             LumpToken::MapMarker("E1M1".to_string()),
//             LumpToken::Lump("THINGS".to_string(), LumpRef::new(0, 10, 0)),
//             LumpToken::Lump("LINEDEFS".to_string(), LumpRef::new(10, 20, 10)),
//             LumpToken::Lump("SIDEDEFS".to_string(), LumpRef::new(20, 30, 20)),
//             LumpToken::Lump("VERTEXES".to_string(), LumpRef::new(30, 40, 30)),
//             LumpToken::Lump("SECTORS".to_string(), LumpRef::new(40, 50, 40)),
//             LumpToken::Lump("SSECTORS".to_string(), LumpRef::new(50, 50, 50)),
//             LumpToken::Lump("SEGS".to_string(), LumpRef::new(50, 50, 50)),
//             LumpToken::Lump("NODES".to_string(), LumpRef::new(50, 50, 50)),
//             LumpToken::Lump("REJECT".to_string(), LumpRef::new(50, 50, 50)),
//             LumpToken::Lump("BLOCKMAP".to_string(), LumpRef::new(50, 50, 50)),
//             LumpToken::Lump("BEHAVIOR".to_string(), LumpRef::new(50, 50, 50)),
//         ]);
//         let mut map_iterator = MapIterator::new(Rc::clone(&tokens));
//         let map = map_iterator.next().unwrap();
//         assert_eq!(map.name(), "E1M1");
//         assert_eq!(map.things().unwrap().range(), (0, 10));
//         assert_eq!(map.linedefs().unwrap().range(), (10, 20));
//         assert_eq!(map.sidedefs().unwrap().range(), (20, 30));
//         assert_eq!(map.vertexes().unwrap().range(), (30, 40));
//         assert_eq!(map.sectors().unwrap().range(), (40, 50));
//         assert_eq!(map.ssectors().unwrap().range(), (50, 50));
//         assert_eq!(map.segs().unwrap().range(), (50, 50));
//         assert_eq!(map.nodes().unwrap().range(), (50, 50));
//         assert_eq!(map.rejects().unwrap().range(), (50, 50));
//         assert_eq!(map.blockmap().unwrap().range(), (50, 50));
//
//         let next_map = map_iterator.next();
//         assert!(next_map.is_none());
//     }
//
//     #[test]
//     fn map_iterator_handles_multiple_maps() {
//         let tokens = Rc::new(vec![
//             LumpToken::MapMarker("E1M1".to_string()),
//             LumpToken::Lump("THINGS".to_string(), LumpRef::new(0, 10, 0)),
//             LumpToken::Lump("LINEDEFS".to_string(), LumpRef::new(10, 20, 10)),
//             LumpToken::Lump("SIDEDEFS".to_string(), LumpRef::new(20, 30, 20)),
//             LumpToken::Lump("VERTEXES".to_string(), LumpRef::new(30, 40, 30)),
//             LumpToken::Lump("SECTORS".to_string(), LumpRef::new(40, 50, 40)),
//             LumpToken::MapMarker("E1M2".to_string()),
//             LumpToken::Lump("THINGS".to_string(), LumpRef::new(50, 60, 50)),
//             LumpToken::Lump("LINEDEFS".to_string(), LumpRef::new(60, 70, 60)),
//             LumpToken::Lump("SIDEDEFS".to_string(), LumpRef::new(70, 80, 70)),
//             LumpToken::Lump("VERTEXES".to_string(), LumpRef::new(80, 90, 80)),
//             LumpToken::Lump("SECTORS".to_string(), LumpRef::new(90, 100, 90)),
//         ]);
//
//         let mut map_iterator = MapIterator::new(Rc::clone(&tokens));
//         let map1 = map_iterator.next().unwrap();
//         assert_eq!(map1.name(), "E1M1");
//         assert_eq!(map1.things().unwrap().range(), (0, 10));
//         let map2 = map_iterator.next().unwrap();
//         assert_eq!(map2.name(), "E1M2");
//         assert_eq!(map2.things().unwrap().range(), (50, 60));
//         let next_map = map_iterator.next();
//         assert!(next_map.is_none());
//     }
//
//     #[test]
//     fn map_iterator_stops_at_non_map_tokens() {
//         let tokens = Rc::new(vec![
//             LumpToken::MapMarker("E1M1".to_string()),
//             LumpToken::Lump("THINGS".to_string(), LumpRef::new(0, 10, 0)),
//             LumpToken::Lump("LINEDEFS".to_string(), LumpRef::new(10, 20, 10)),
//             LumpToken::Lump("SIDEDEFS".to_string(), LumpRef::new(20, 30, 20)),
//             LumpToken::Lump("VERTEXES".to_string(), LumpRef::new(30, 40, 30)),
//             LumpToken::Lump("SECTORS".to_string(), LumpRef::new(40, 50, 40)),
//             LumpToken::Lump("SND".to_string(), LumpRef::new(50, 60, 50)), // Non-map lump
//             LumpToken::MapMarker("E1M1".to_string()),
//         ]);
//
//         let mut map_iterator = MapIterator::new(Rc::clone(&tokens));
//         let map = map_iterator.next().unwrap();
//         assert_eq!(map.name(), "E1M1");
//         assert_eq!(map.things().unwrap().range(), (0, 10));
//         let next_map = map_iterator.next();
//         assert!(next_map.is_none());
//         let next_map_after = map_iterator.next();
//         assert!(next_map_after.is_none());
//     }
// }
