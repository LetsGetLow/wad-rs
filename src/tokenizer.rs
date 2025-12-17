use crate::directory::DirectoryIterator;
use crate::lumps::LumpRef;

#[derive(Debug, Clone)]
pub enum LumpToken {
    MarkerStart(String),
    MarkerEnd(String),
    MapMarker(String),
    Lump(String, LumpRef),
}

impl LumpToken {
    pub fn is_start_marker(name: &str) -> bool {
        name.ends_with("_START")
    }

    pub fn is_end_marker(name: &str) -> bool {
        name.ends_with("_END")
    }
}
pub unsafe fn tokenize_lumps(directory_iterator: DirectoryIterator, data: &[u8]) -> Vec<LumpToken> {
    let mut tokens = Vec::new();

    for dir_ref in directory_iterator {
        let name = unsafe { dir_ref.name(&data) };
        if dir_ref.is_marker() {
            let name = unsafe { dir_ref.name(&data) };
            if is_map_marker(&name) {
                tokens.push(LumpToken::MapMarker(name));
            } else if LumpToken::is_start_marker(&name) {
                tokens.push(LumpToken::MarkerStart(name));
            } else if LumpToken::is_end_marker(&name) {
                tokens.push(LumpToken::MarkerEnd(name));
            }
        } else {
            tokens.push(LumpToken::Lump(name, dir_ref));
        }
    }

    tokens
}

fn is_map_marker(name: &str) -> bool {
    match name.as_bytes() {
        [b'M', b'A', b'P', d1, d2] => d1.is_ascii_digit() && d2.is_ascii_digit(),
        [b'E', d1, b'M', d2] => d1.is_ascii_digit() && d2.is_ascii_digit(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn tokenize_lumps_produces_correct_start_end_marker_tokens() {
        let data= Rc::from(vec![
            // _START marker
            0, 0, 0, 0, 0, 0, 0, 0, b'_', b'S', b'T', b'A', b'R', b'T', 0, 0, // _END marker
            0, 0, 0, 0, 0, 0, 0, 0, b'_', b'E', b'N', b'D', 0, 0, 0, 0,
        ]);

        let dir_iterator = DirectoryIterator::seed_test_data(Rc::clone(&data), 0, 32);
        let tokens = unsafe { tokenize_lumps(dir_iterator, &data) };

        assert_eq!(tokens.len(), 2);
        match &tokens[0] {
            LumpToken::MarkerStart(name) => assert_eq!(name, "_START"),
            _ => panic!("Expected MarkerStart token"),
        }
        match &tokens[1] {
            LumpToken::MarkerEnd(name) => assert_eq!(name, "_END"),
            _ => panic!("Expected MarkerEnd token"),
        }
    }

    #[test]
    fn tokenize_lumps_produces_correct_lump_tokens() {
        let data= Rc::from(vec![
            // LUMP1
            4, 0, 0, 0, 1, 0, 0, 0, b'L', b'U', b'M', b'P', b'1', 0, 0, 0, // LUMP2
            3, 0, 0, 0, 2, 0, 0, 0, b'L', b'U', b'M', b'P', b'2', 0, 0, 0,
        ]);

        let dir_iterator = DirectoryIterator::seed_test_data(Rc::clone(&data), 0, 32);
        let tokens = unsafe { tokenize_lumps(dir_iterator, &data) };

        assert_eq!(tokens.len(), 2);
        match &tokens[0] {
            LumpToken::Lump(name, dref) => {
                assert_eq!(name, "LUMP1");
                assert_eq!(dref.start(), 4);
                assert_eq!(dref.end(), 5);
            }
            _ => panic!("Expected Lump token for LUMP1"),
        }
        match &tokens[1] {
            LumpToken::Lump(name, dref) => {
                assert_eq!(name, "LUMP2");
                assert_eq!(dref.start(), 3);
                assert_eq!(dref.end(), 5);
            }
            _ => panic!("Expected Lump token for LUMP2"),
        }
    }

    #[test]
    fn tokenize_lumps_produces_map_marker_tokens() {
        let data= Rc::from(vec![
            // MAP01 marker Doom2, Heretic style
            0, 0, 0, 0, 0, 0, 0, 0, b'M', b'A', b'P', b'0', b'1', 0, 0, 0,
            // E1M2 marker Doom style
            0, 0, 0, 0, 0, 0, 0, 0, b'E', b'1', b'M', b'2', 0, 0, 0, 0,
        ]);
        let dir_iterator = DirectoryIterator::seed_test_data(Rc::clone(&data), 0, 32);
        let tokens = unsafe { tokenize_lumps(dir_iterator, &data) };

        assert_eq!(tokens.len(), 2);
        match &tokens[0] {
            LumpToken::MapMarker(name) => assert_eq!(name, "MAP01"),
            _ => panic!("Expected MapMarker token for MAP01"),
        }
        match &tokens[1] {
            LumpToken::MapMarker(name) => assert_eq!(name, "E1M2"),
            _ => panic!("Expected MapMarker token for E1M2"),
        }
    }
}
