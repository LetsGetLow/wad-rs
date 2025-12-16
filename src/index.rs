use crate::lumps::LumpCollection;
use crate::tokenizer::LumpToken;
use std::iter::Peekable;
use std::slice::Iter;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

pub fn parse_tokens(tokens: Vec<LumpToken>) -> Result<LumpCollection> {
    let mut lumps = LumpCollection::default();
    let mut map_collection = LumpCollection::default();
    let mut iter = tokens.iter().peekable();

    while let Some(token) = iter.peek() {
        match token {
            LumpToken::Lump(name, dir_reference) => {
                lumps.add_lump(name, *dir_reference)?;
            }
            LumpToken::MarkerStart(name) => {
                let namespace = name.replace("_START", "");
                lumps.add_collection(&name, parse_namespace(&namespace, &mut iter)?)?;
            }
            LumpToken::MapMarker(name) => {
                map_collection.add_collection(name, parse_map(&mut iter)?)?;
                // immediately continue to avoid iter.next() below because maps do not have end markers
                continue;
            }
            _ => {}
        }
        iter.next();
    }

    lumps.add_collection("MAPS", map_collection)?;

    Ok(lumps)
}

fn parse_namespace(namespace: &str, iter: &mut Peekable<Iter<LumpToken>>) -> Result<LumpCollection> {
    iter.next();
    let mut namespace_map = LumpCollection::default();

    while let Some(token) = iter.peek() {
        match token {
            LumpToken::Lump(name, dir_reference) => {
                namespace_map.add_lump(name, *dir_reference)?;
            }
            LumpToken::MarkerStart(name) => {
                let namespace = name.replace("_START", "");
                namespace_map.add_collection(&name, parse_namespace(&namespace, iter)?)?;
            }
            LumpToken::MarkerEnd(name) => {
                let namespace_end = name.replace("_END", "");
                if namespace != namespace_end {
                    return Err(format!(
                        "Mismatched end marker: expected namespace '{}', found '{}'",
                        namespace,
                        namespace_end
                    ).into());
                }
                break;
            }
            _ => {}
        }
        iter.next();
    }
    Ok(namespace_map)
}

fn parse_map(iter: &mut Peekable<Iter<LumpToken>>) -> Result<LumpCollection> {
    iter.next();
    let mut map_collection = LumpCollection::default();
    while let Some(token) = iter.peek() {
        match token {
            LumpToken::MapMarker(_) => break,
            LumpToken::Lump(name, dir_ref) => {
                let is_map_lump = matches!(
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
                    );
                if is_map_lump {
                    map_collection.add_lump(name, *dir_ref)?;
                } else {
                    break;
                }
            }
            _ => break,
        };
        iter.next();
    }

    Ok(map_collection)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::DirectoryRef;
    use crate::tokenizer::LumpToken;

    #[test]
    fn parse_token_can_detect_lumps() {
        let tokens = vec![
            LumpToken::Lump("LUMP1".to_string(), DirectoryRef::new(0, 10, 0)),
            LumpToken::Lump("LUMP2".to_string(), DirectoryRef::new(10, 20, 10)),
        ];

        let result = parse_tokens(tokens).unwrap();
        assert_eq!(result.get_lump("LUMP1"), Some(&DirectoryRef::new(0, 10, 0)));
        assert_eq!(result.get_lump("LUMP2"), Some(&DirectoryRef::new(10, 20, 10)));
    }

    #[test]
    fn parse_token_handles_empty_namespaces() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MarkerStart("S_START".to_string()),
            LumpToken::MarkerEnd("S_END".to_string()),
        ];

        let result = parse_tokens(tokens).unwrap();

        assert!(!result.has_lumps());
        assert!(result.has_collections());

        let collection = result.get_collection("S_START").unwrap();
        assert!(!collection.has_lumps());
        assert!(!collection.has_collections());
    }

    #[test]
    fn parse_token_detects_nested_namespaces() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MarkerStart("N_START".to_string()),
            LumpToken::Lump("N_LUMP".to_string(), DirectoryRef::new(0, 10, 0)),
            LumpToken::MarkerEnd("N_END".to_string()),
        ];

        let result = parse_tokens(tokens).unwrap();

        assert!(!result.has_lumps());
        assert!(result.has_collections());

        let collection = result.get_collection("N_START").unwrap();
        assert_eq!(collection.get_lump("N_LUMP"), Some(&DirectoryRef::new(0, 10, 0)));
    }

    #[test]
    fn parse_token_detects_invalid_end_marker() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MarkerStart("X_START".to_string()),
            LumpToken::MarkerEnd("Y_END".to_string()),
        ];

        let result = parse_tokens(tokens);

        println!("Error: {}", result.as_ref().err().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn parse_token_detects_multiple_nested_namespaces() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MarkerStart("OUTER_START".to_string()),
            LumpToken::Lump("OUTER_LUMP".to_string(), DirectoryRef::new(0, 10, 0)),
            LumpToken::MarkerStart("INNER_START".to_string()),
            LumpToken::Lump("INNER_LUMP".to_string(), DirectoryRef::new(10, 20, 10)),
            LumpToken::MarkerEnd("INNER_END".to_string()),
            LumpToken::MarkerEnd("OUTER_END".to_string()),
        ];

        let result = parse_tokens(tokens).unwrap();

        assert!(!result.has_lumps());
        assert!(result.has_collections());
        let outer = result.get_collection("OUTER_START").unwrap();
        assert_eq!(outer.get_lump("OUTER_LUMP"), Some(&DirectoryRef::new(0, 10, 0)));
        let inner = outer.get_collection("INNER_START").unwrap();
        assert_eq!(inner.get_lump("INNER_LUMP"), Some(&DirectoryRef::new(10, 20, 10)));
    }

    #[test]
    fn parse_token_can_detect_doom1_maps() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MapMarker("E1M1".to_string()),
        ];
        let result = parse_tokens(tokens).unwrap();
        let map_collection = result.get_collection("MAPS").unwrap();
        let map = map_collection.get_collection("E1M1").unwrap();

        assert!(!result.has_lumps());
        assert!(!map_collection.has_lumps());
        assert!(map_collection.has_collections());
        assert!(!map.has_collections());
        assert!(!map.has_lumps());
    }

    #[test]
    fn parse_token_can_detect_doom2_maps() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MapMarker("MAP01".to_string()),
        ];
        let result = parse_tokens(tokens).unwrap();
        let map_collection = result.get_collection("MAPS").unwrap();
        let map = map_collection.get_collection("MAP01").unwrap();

        assert!(!result.has_lumps());
        assert!(!map_collection.has_lumps());
        assert!(map_collection.has_collections());
        assert!(!map.has_collections());
        assert!(!map.has_lumps());
    }

    #[test]
    fn parse_tokens_can_detect_map_boundaries() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MapMarker("E1M1".to_string()),
            LumpToken::Lump("THINGS".to_string(), DirectoryRef::new(0, 10, 0)),
            LumpToken::MapMarker("E1M2".to_string()),
            LumpToken::Lump("THINGS".to_string(), DirectoryRef::new(10, 20, 10)),
            LumpToken::Lump("SND".to_string(), DirectoryRef::new(10, 20, 10)),
        ];

        let result = parse_tokens(tokens).unwrap();
        let map_collection = result.get_collection("MAPS").unwrap();
        let map1 = map_collection.get_collection("E1M1").unwrap();
        let map2 = map_collection.get_collection("E1M2").unwrap();

        let snd = result.iter().next().unwrap();
        assert_eq!(snd.0.as_str(), "SND");
        assert_eq!(map1.get_lump("THINGS"), Some(&DirectoryRef::new(0, 10, 0)));
        assert_eq!(map2.get_lump("THINGS"), Some(&DirectoryRef::new(10, 20, 10)));
    }

    #[test]
    fn parse_map_can_parse_map_lumps() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MapMarker("E1M1".to_string()),
            LumpToken::Lump("THINGS".to_string(), DirectoryRef::new(0, 10, 0)),
            LumpToken::Lump("LINEDEFS".to_string(), DirectoryRef::new(10, 20, 10)),
            LumpToken::Lump("SIDEDEFS".to_string(), DirectoryRef::new(20, 30, 20)),
            LumpToken::Lump("VERTEXES".to_string(), DirectoryRef::new(30, 40, 30)),
            LumpToken::Lump("SECTORS".to_string(), DirectoryRef::new(40, 50, 40)),
            LumpToken::Lump("SSECTORS".to_string(), DirectoryRef::new(50, 60, 50)),
            LumpToken::Lump("SEGS".to_string(), DirectoryRef::new(60, 70, 60)),
            LumpToken::Lump("NODES".to_string(), DirectoryRef::new(70, 80, 70)),
            LumpToken::Lump("BLOCKMAP".to_string(), DirectoryRef::new(80, 90, 80)),
            LumpToken::Lump("REJECT".to_string(), DirectoryRef::new(90, 100, 100)),
        ];

        let mut iter = tokens.iter().peekable();
        let map_collection = parse_map(&mut iter).unwrap();
        assert_eq!(map_collection.get_lump("THINGS"), Some(&DirectoryRef::new(0, 10, 0)));
        assert_eq!(map_collection.get_lump("LINEDEFS"), Some(&DirectoryRef::new(10, 20, 10)));
        assert_eq!(map_collection.get_lump("SIDEDEFS"), Some(&DirectoryRef::new(20, 30, 20)));
        assert_eq!(map_collection.get_lump("VERTEXES"), Some(&DirectoryRef::new(30, 40, 30)));
        assert_eq!(map_collection.get_lump("SECTORS"), Some(&DirectoryRef::new(40, 50, 40)));
        assert_eq!(map_collection.get_lump("SSECTORS"), Some(&DirectoryRef::new(50, 60, 50)));
        assert_eq!(map_collection.get_lump("SEGS"), Some(&DirectoryRef::new(60, 70, 60)));
        assert_eq!(map_collection.get_lump("NODES"), Some(&DirectoryRef::new(70, 80, 70)));
        assert_eq!(map_collection.get_lump("BLOCKMAP"), Some(&DirectoryRef::new(80, 90, 80)));
        assert_eq!(map_collection.get_lump("REJECT"), Some(&DirectoryRef::new(90, 100, 100)));
    }

    #[test]
    fn parse_map_stops_on_non_map_lumps() {
        let tokens: Vec<LumpToken> = vec![
            LumpToken::MapMarker("E1M1".to_string()),
            LumpToken::Lump("THINGS".to_string(), DirectoryRef::new(0, 10, 0)),
            LumpToken::Lump("LINEDEFS".to_string(), DirectoryRef::new(10, 20, 10)),
            LumpToken::MapMarker("E1M2".to_string()),
        ];

        let mut iter = tokens.iter().peekable();
        let map_collection = parse_map(&mut iter).unwrap();


        let len = map_collection.iter().len();
        assert_eq!(len, 2);
    }
}
