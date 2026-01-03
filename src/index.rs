use crate::lump::LumpRef;
use crate::tokenizer::{LumpToken, TokenIterator};
use std::collections::HashMap;
use std::iter::Peekable;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub enum LumpNode<'a> {
    Namespace {
        name: &'a str,
        children: HashMap<&'a str, LumpNode<'a>>,
    },
    Lump {
        name: &'a str,
        lump: LumpRef<'a>,
    },
}

impl<'a> LumpNode<'a> {
    pub fn namespace(name: &'a str, children: HashMap<&'a str, LumpNode<'a>>) -> Self {
        LumpNode::Namespace { name, children }
    }

    pub fn lump(name: &'a str, lump: LumpRef<'a>) -> Self {
        LumpNode::Lump { name, lump }
    }
}

pub fn index_tokens<'a>(tokens: TokenIterator<'a>) -> Result<HashMap<&'a str, LumpNode<'a>>> {
    let mut tokens = tokens.peekable();
    let mut lumps: HashMap<&'a str, LumpNode<'a>> = HashMap::new();
    let mut maps: HashMap<&'a str, LumpNode<'a>> = HashMap::new();

    while let Some(result) = tokens.next() {
        let token = result?;
        match token {
            LumpToken::Lump(name, lump_ref) => {
                let lump_node = LumpNode::lump(name, lump_ref);
                lumps.insert(name, lump_node);
            }

            LumpToken::MapMarker(name) => {
                let map = index_map(name, &mut tokens)?;
                maps.insert(name, map);
                continue;
            }

            LumpToken::MarkerStart(marker) => {
                let children = index_namespace(marker, &mut tokens)?;
                let namespace_node = LumpNode::namespace(marker, children);
                lumps.insert(marker, namespace_node);
            }
            LumpToken::MarkerEnd(_) => {
                return Err("Unexpected end marker without matching start marker".into());
            }
        }
    }
    lumps.insert("MAPS", LumpNode::namespace("MAPS", maps));

    Ok(lumps)
}

fn index_map<'a>(name: &'a str, tokens: &mut Peekable<TokenIterator<'a>>) -> Result<LumpNode<'a>> {
    tokens.next();

    let mut map = HashMap::new();
    while let Some(Ok(LumpToken::Lump(name, ..))) = tokens.peek() {
        match *name {
            "THINGS" | "LINEDEFS" | "SIDEDEFS" | "VERTEXES" | "SECTORS" | "SEGS" | "SSECTORS"
            | "NODES" | "REJECT" | "BLOCKMAP" | "BEHAVIOR" => {
                if let Some(Ok(LumpToken::Lump(name, lump_ref))) = tokens.next() {
                    map.insert(name, LumpNode::lump(name, lump_ref));
                }
            }
            _ => break,
        }
    }

    Ok(LumpNode::namespace(name, map))
}

fn index_namespace<'a>(
    namespace: &'a str,
    tokens: &mut Peekable<TokenIterator<'a>>,
) -> Result<HashMap<&'a str, LumpNode<'a>>> {
    let mut lumps = HashMap::new();

    while let Some(result) = tokens.next() {
        let token = result?;

        match token {
            LumpToken::Lump(name, lump_ref) => {
                lumps.insert(name, LumpNode::lump(name, lump_ref));
            }

            LumpToken::MarkerStart(name) => {
                let children = index_namespace(name, tokens)?;
                lumps.insert(name, LumpNode::namespace(name, children));
            }

            LumpToken::MarkerEnd(name) => {
                let end_ns = name
                    .strip_suffix("_END")
                    .ok_or_else(|| format!("Invalid end marker name: {}", name))?;

                let start_ns = namespace
                    .strip_suffix("_START")
                    .ok_or_else(|| format!("Invalid start marker name: {}", namespace))?;

                return if start_ns == end_ns {
                    Ok(lumps)
                } else {
                    Err(format!(
                        "Mismatched end marker: expected '{}', found '{}'",
                        start_ns, end_ns
                    )
                    .into())
                };
            }

            _ => {}
        }
    }

    Ok(lumps)
}

// fn index_namespace<'a>(
//     namespace: &'a str,
//     tokens: &mut Peekable<TokenIterator>,
// ) -> Result<HashMap<&'a str, LumpNode<'a>>> {
//     let mut lumps: HashMap<&str, LumpNode> = HashMap::new();
//     tokens.next();
//     while let Some(token) = tokens.peek() {
//         if let Err(_err) = token {
//             // TODO: improve error handling
//             return Err("Error while indexing namespace".into());
//         }
//         let token = token.as_ref().unwrap();
//         match token {
//             LumpToken::Lump(name, lump_ref) => {
//                 let lump_node = LumpNode::lump(name, lump_ref);
//                 lumps.insert(name, lump_node);
//
//             }
//             LumpToken::MarkerStart(name) => {
//                 let children = index_namespace(name, tokens)?;
//                 let namespace_node = LumpNode::Namespace {
//                     name,
//                     children,
//                 };
//                 lumps.insert(name, namespace_node);
//             }
//             LumpToken::MarkerEnd(name) => {
//                 let end_ns = name.strip_suffix("_END").ok_or_else(|| {;
//                     format!("Invalid end marker name: expected to end with '_END', found '{}'", name)
//                 })?;
//                 let start_ns = namespace.strip_suffix("_START").ok_or_else(|| {
//                     format!("Invalid start marker name: expected to end with '_START', found '{}'", namespace)
//                 })?;
//
//                 if start_ns == end_ns {
//                     break;
//                 }
//                 return Err(format!(
//                     "Mismatched end marker: expected namespace '{}', found '{}'",
//                     start_ns, end_ns
//                 )
//                     .into());
//             }
//             _ => {}
//         }
//         tokens.next();
//     }
//     Ok(lumps)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::lump::LumpRef;
//     use crate::tokenizer::LumpToken;
//
//     #[test]
//     fn index_tokens_can_index_lumps() {
//         let tokens = vec![
//             LumpToken::Lump("LUMP1".to_string(), LumpRef::new(0, 10, 0)),
//             LumpToken::Lump("LUMP2".to_string(), LumpRef::new(10, 20, 10)),
//         ];
//
//         let result = index_tokens(&tokens).unwrap();
//         assert_eq!(result.get("LUMP1"), Some(&LumpRef::new(0, 10, 0)));
//         assert_eq!(result.get("LUMP2"), Some(&LumpRef::new(10, 20, 10)));
//     }
//
//     #[test]
//     fn index_tokens_skips_map_lumps() {
//         let tokens = vec![
//             LumpToken::MapMarker("E1M1".to_string()),
//             LumpToken::Lump("THINGS".to_string(), LumpRef::new(0, 10, 0)),
//             LumpToken::Lump("LINEDEFS".to_string(), LumpRef::new(10, 20, 10)),
//             LumpToken::Lump("SIDEDEFS".to_string(), LumpRef::new(20, 30, 20)),
//             LumpToken::Lump("VERTEXES".to_string(), LumpRef::new(30, 40, 30)),
//             LumpToken::Lump("SEGS".to_string(), LumpRef::new(60, 70, 60)),
//             LumpToken::Lump("SSECTORS".to_string(), LumpRef::new(50, 60, 50)),
//             LumpToken::Lump("NODES".to_string(), LumpRef::new(70, 80, 70)),
//             LumpToken::Lump("SECTORS".to_string(), LumpRef::new(40, 50, 40)),
//             LumpToken::Lump("REJECT".to_string(), LumpRef::new(90, 100, 100)),
//             LumpToken::Lump("BLOCKMAP".to_string(), LumpRef::new(80, 90, 80)),
//             LumpToken::Lump("BEHAVIOR".to_string(), LumpRef::new(100, 110, 110)),
//             LumpToken::Lump("SND".to_string(), LumpRef::new(20, 30, 20)),
//         ];
//
//         let result = index_tokens(&tokens).unwrap();
//         assert_eq!(result.len(), 1);
//         assert_eq!(result.get("SND"), Some(&LumpRef::new(20, 30, 20)));
//     }
//
//     #[test]
//     fn index_tokens_can_index_namespaced_lumps() {
//         let tokens = vec![
//             LumpToken::MarkerStart("S_START".to_string()),
//             LumpToken::Lump("LUMP".to_string(), LumpRef::new(0, 10, 0)),
//             LumpToken::MarkerEnd("S_END".to_string()),
//             LumpToken::Lump("LUMP".to_string(), LumpRef::new(10, 20, 10)),
//         ];
//
//         let result = index_tokens(&tokens).unwrap();
//         assert_eq!(result.get("S/LUMP"), Some(&LumpRef::new(0, 10, 0)));
//         assert_eq!(result.get("LUMP"), Some(&LumpRef::new(10, 20, 10)));
//     }
//
//     #[test]
//     fn index_tokens_detects_nested_namespaces() {
//         let tokens = vec![
//             LumpToken::MarkerStart("OUTER_START".to_string()),
//             LumpToken::Lump("OUTER_LUMP".to_string(), LumpRef::new(0, 10, 0)),
//             LumpToken::MarkerStart("INNER_START".to_string()),
//             LumpToken::Lump("INNER_LUMP".to_string(), LumpRef::new(10, 20, 10)),
//             LumpToken::MarkerEnd("INNER_END".to_string()),
//             LumpToken::MarkerEnd("OUTER_END".to_string()),
//         ];
//
//         let result = index_tokens(&tokens).unwrap();
//         assert_eq!(
//             result.get("OUTER/OUTER_LUMP"),
//             Some(&LumpRef::new(0, 10, 0))
//         );
//         assert_eq!(
//             result.get("OUTER/INNER/INNER_LUMP"),
//             Some(&LumpRef::new(10, 20, 10))
//         );
//     }
//
//     #[test]
//     fn index_tokens_detects_invalid_end_marker() {
//         let tokens = vec![
//             LumpToken::MarkerStart("X_START".to_string()),
//             LumpToken::MarkerEnd("Y_END".to_string()),
//         ];
//         let result = index_tokens(&tokens);
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn index_tokens_can_detect_dangling_end_marker() {
//         let tokens = vec![
//             LumpToken::Lump("LUMP1".to_string(), LumpRef::new(0, 10, 0)),
//             LumpToken::MarkerEnd("S_END".to_string()),
//         ];
//         let result = index_tokens(&tokens);
//         assert!(result.is_err());
//     }
// }
