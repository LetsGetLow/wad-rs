use crate::lump::LumpRef;
use crate::tokenizer::{LumpToken, TokenIterator};
use std::collections::HashMap;
use std::iter::Peekable;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
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

    // maps namespace will always be created even if empty
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
                        "Dangling end marker: expected '{}', found '{}'",
                        start_ns, end_ns
                    )
                    .into())
                };
            }

            _ => {}
        }
    }

    // should never reach here
    Err(format!(
        "Dangling start marker: no matching end marker for '{}'",
        namespace
    ).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lump::LumpRef;
    use crate::tokenizer::fake_token_data;

    #[test]
    fn index_tokens_can_index_lumps() {
        let (header, data) = fake_token_data(&[("LUMP1", &[0, 0]), ("LUMP2", &[1, 1])]);
        let tokens = TokenIterator::new(header, &data).unwrap();

        let index = index_tokens(tokens).unwrap();

        assert_eq!(index.len(), 3); // two lumps plus the MAPS namespace
        assert_eq!(
            index.get("LUMP1"),
            Some(LumpNode::Lump {
                name: "LUMP1",
                lump: LumpRef::new("LUMP1", &[0, 0])
            })
            .as_ref()
        );
        assert_eq!(
            index.get("LUMP2"),
            Some(LumpNode::Lump {
                name: "LUMP2",
                lump: LumpRef::new("LUMP2", &[1, 1])
            })
            .as_ref()
        );
    }

    #[test]
    fn index_tokens_skips_map_lumps() {
        let (header, data) = crate::tokenizer::fake_token_data(&[
            ("E1M1", &[]),
            ("THINGS", &[0; 10]),
            ("LINEDEFS", &[1; 10]),
            ("SIDEDEFS", &[2; 10]),
            ("VERTEXES", &[3; 10]),
            ("SECTORS", &[4; 10]),
            ("SEGS", &[5; 10]),
            ("SSECTORS", &[6; 10]),
            ("NODES", &[7; 10]),
            ("REJECT", &[8; 10]),
            ("BLOCKMAP", &[9; 10]),
            ("BEHAVIOR", &[10; 10]),
            ("SND", &[11; 10]),
        ]);
        let tokens = TokenIterator::new(header, &data).unwrap();
        let index = index_tokens(tokens).unwrap();
        assert_eq!(index.len(), 2); // one for the maps and one for SND
        assert_eq!(
            index.get("SND").unwrap(),
            &LumpNode::Lump {
                name: "SND",
                lump: LumpRef::new("SND", &[11; 10])
            }
        );
    }

    #[test]
    fn index_tokens_can_index_namespaced_lumps() {
        let (header, data) = crate::tokenizer::fake_token_data(&[
            ("S_START", &[]),
            ("LUMP", &[0; 10]),
            ("S_END", &[]),
            ("LUMP", &[1; 10]),
        ]);

        let tokens = TokenIterator::new(header, &data).unwrap();

        let index = index_tokens(tokens).unwrap();
        let ns = match index.get("S_START") {
            Some(LumpNode::Namespace { children, .. }) => children,
            _ => panic!("Expected S_START to be a namespace"),
        };

        assert_eq!(
            ns.get("LUMP").unwrap(),
            &LumpNode::lump("LUMP", LumpRef::new("LUMP", &[0; 10]))
        );
        assert_eq!(
            index.get("LUMP").unwrap(),
            &LumpNode::lump("LUMP", LumpRef::new("LUMP", &[1; 10]))
        );
    }

    #[test]
    fn index_tokens_detects_nested_namespaces() {
        let (header, data) = fake_token_data(&[
            ("O_START", &[]),
            ("O_LUMP", &[0; 10]),
            ("I_START", &[]),
            ("I_LUMP", &[10; 20]),
            ("I_END", &[]),
            ("O_END", &[]),
        ]);

        let tokens = TokenIterator::new(header, &data).unwrap();

        let index = index_tokens(tokens).unwrap();
        let outer_ns = match index.get("O_START") {
            Some(LumpNode::Namespace { children, .. }) => children,
            _ => panic!("Expected O_START to be a namespace"),
        };
        assert_eq!(
            outer_ns.get("O_LUMP").unwrap(),
            &LumpNode::lump("O_LUMP", LumpRef::new("O_LUMP", &[0; 10]))
        );

        let inner_ns = match outer_ns.get("I_START") {
            Some(LumpNode::Namespace { children, .. }) => children,
            _ => panic!("Expected I_START to be a namespace"),
        };

        assert_eq!(
            inner_ns.get("I_LUMP").unwrap(),
            &LumpNode::lump("I_LUMP", LumpRef::new("I_LUMP", &[10; 20])),
        );
    }

    #[test]
    fn index_tokens_detects_invalid_end_marker() {
        let (header, data) = fake_token_data(&[("X_START", &[]), ("Y_END", &[])]);
        let tokens = TokenIterator::new(header, &data).unwrap();
        let result = index_tokens(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn index_tokens_can_detect_dangling_end_marker() {
        let (header, data) = fake_token_data(&[("LUMP1", &[0;10]), ("S_END", &[])]);
        let tokens = TokenIterator::new(header, &data).unwrap();
        let result = index_tokens(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn index_tokens_can_detect_dangling_start_marker() {
        let (header, data) = fake_token_data(&[("S_START", &[])]);
        let tokens = TokenIterator::new(header, &data).unwrap();
        let result = index_tokens(tokens);
        assert!(result.is_err());
    }
}
