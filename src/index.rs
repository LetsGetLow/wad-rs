use crate::lump::is_map_lump;
use crate::lump::LumpRef;
use crate::tokenizer::{LumpToken, TokenIterator};
use std::collections::HashMap;
use std::iter::Peekable;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub fn index_tokens(tokens: TokenIterator) -> Result<HashMap<String, LumpRef>> {
    let mut tokens = tokens.peekable();
    let mut lumps = HashMap::new();

    while let Some(result) = tokens.peek() {

        let token = result.as_ref().unwrap();
        match token {
            LumpToken::Lump(name, lump_ref) => {
                lumps.insert(name.to_string(), *lump_ref);
            }

            LumpToken::MapMarker(_) => {
                skip_map_lumps(&mut tokens);
                continue;
            }

            LumpToken::MarkerStart(marker) => {
                let namespace = marker.replace("_START", "");
                index_namespace(&mut lumps, &namespace, &mut tokens)?;
            }
            LumpToken::MarkerEnd(_) => {
                return Err("Unexpected end marker without matching start marker".into());
            }
        }
        tokens.next();
    }

    Ok(lumps)
}

fn skip_map_lumps(tokens: &mut Peekable<TokenIterator>) {
    tokens.next();
    while let Some(token) = tokens.peek() {
        if let Err(_) = token {
            continue;
        }
        let token = token.as_ref().unwrap();
        match token {
            LumpToken::Lump(name, _) => {
                if !is_map_lump(name) {
                    break;
                }
            }
            _ => { break; }
        }
        tokens.next();
    }
}

fn index_namespace(
    lumps: &mut HashMap<String, LumpRef>,
    namespace: &String,
    tokens: &mut Peekable<TokenIterator>,
) -> Result<()> {
    tokens.next();
    while let Some(token) = tokens.peek() {
        if let Err(_err) = token {
            // TODO: improve error handling
            return Err("Error while indexing namespace".into());
        }
        let token = token.as_ref().unwrap();
        match token {
            LumpToken::Lump(name, lump_ref) => {
                let namespaced_name = format!("{}/{}", namespace, name);
                lumps.insert(namespaced_name, *lump_ref);
            }
            LumpToken::MarkerStart(start_marker) => {
                let inner_namespace = start_marker.replace("_START", "");
                let mut full_namespace = String::with_capacity(namespace.len() + 1 + inner_namespace.len());
                full_namespace.push_str(namespace);
                full_namespace.push('/');
                full_namespace.push_str(&inner_namespace);
                index_namespace(lumps, &full_namespace, tokens)?;
            }
            LumpToken::MarkerEnd(end_marker) => {
                let namespace_end = end_marker.replace("_END", "");
                if *namespace == namespace_end || namespace.ends_with(&namespace_end) {
                    break;
                }
                return Err(format!(
                    "Mismatched end marker: expected namespace '{}', found '{}'",
                    namespace, namespace_end
                )
                .into());
            }
            _ => {}
        }
        tokens.next();
    }
    Ok(())
}

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
